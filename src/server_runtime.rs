use super::{
    parser::Parser, pull_resp, pull_resp::TxPk, Down, MacAddress, Packet, SerializablePacket, Up,
};
use std::{collections::HashMap, net::SocketAddr};
use tokio::net::udp::{RecvHalf, SendHalf};
use tokio::net::UdpSocket;
use tokio::sync::{
    broadcast,
    mpsc::{self, Receiver, Sender},
};

#[derive(Debug)]
enum UdpMessage {
    PacketByMac((Packet, MacAddress)),
    PacketBySocket((Packet, SocketAddr)),
    Client((MacAddress, SocketAddr)),
}

type Request = (Packet, MacAddress);

#[derive(Debug, Clone)]
pub enum Event {
    Packet(Up),
    NewClient((MacAddress, SocketAddr)),
    UpdateClient((MacAddress, SocketAddr)),
    UnableToParseUdpFrame(Vec<u8>),
}

// receives requests from clients
// dispatches them to UdpTx
struct ClientRx {
    sender: Sender<Request>,
    receiver: broadcast::Receiver<Event>,
}

// sends packets to clients
// broadcast enables many clients
type ClientTx = broadcast::Receiver<Event>;

// translates message type such as to restrict
// public message
struct ClientRxTranslator {
    receiver: Receiver<Request>,
    udp_tx_sender: Sender<UdpMessage>,
}

// receives UDP packets
struct UdpRx {
    socket_receiver: RecvHalf,
    udp_tx_sender: Sender<UdpMessage>,
    client_tx_sender: broadcast::Sender<Event>,
}

// transmits UDP packets
struct UdpTx {
    receiver: Receiver<UdpMessage>,
    client_tx_sender: broadcast::Sender<Event>,
    clients: HashMap<MacAddress, SocketAddr>,
    socket_sender: SendHalf,
}

pub struct UdpRuntime {
    tx: ClientTx,
    rx: ClientRx,
}
use rand::Rng;

impl ClientRx {
    pub async fn send(
        &mut self,
        txpk: TxPk,
        mac: MacAddress,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // assign random token
        let random_token = rand::thread_rng().gen();

        // create pull_resp frame with the data
        let packet = pull_resp::Packet {
            random_token,
            data: pull_resp::Data::from_txpk(txpk),
        };
        // send it to UdpTx channel
        self.sender.send((packet.into(), mac)).await?;

        // loop over responses until the TxAck is received
        loop {
            if let Event::Packet(packet) = self.receiver.recv().await? {
                if let Up::TxAck(ack) = packet {
                    if ack.random_token == random_token {
                        return if let Some(error) = ack.get_error() {
                            Err(error.into())
                        } else {
                            Ok(())
                        };
                    }
                }
            }
        }
    }
}

impl UdpRuntime {
    #[allow(dead_code)]
    fn split(self) -> (ClientTx, ClientRx) {
        (self.tx, self.rx)
    }

    pub async fn send(
        &mut self,
        txpk: TxPk,
        mac: MacAddress,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.rx.send(txpk, mac).await
    }

    pub async fn recv(&mut self) -> Result<Event, broadcast::RecvError> {
        self.tx.recv().await
    }

    pub async fn new(addr: SocketAddr) -> Result<UdpRuntime, Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind(&addr).await?;
        let (socket_receiver, socket_sender) = socket.split();

        let (udp_tx_sender, udp_tx_receiver) = mpsc::channel(100);

        // broadcasts to client
        let (client_tx_sender, client_tx_receiver) = broadcast::channel(100);
        // receives requests from clients
        let (client_rx_sender, client_rx_receiver) = mpsc::channel(100);

        let client_rx = ClientRx {
            sender: client_rx_sender,
            receiver: client_tx_sender.subscribe(),
        };

        let client_rx_translator = ClientRxTranslator {
            receiver: client_rx_receiver,
            udp_tx_sender: udp_tx_sender.clone(),
        };

        let client_tx = client_tx_receiver;

        let udp_rx = UdpRx {
            socket_receiver,
            udp_tx_sender,
            client_tx_sender: client_tx_sender.clone(),
        };

        let udp_tx = UdpTx {
            receiver: udp_tx_receiver,
            client_tx_sender,
            clients: HashMap::new(),
            socket_sender,
        };

        // udp_rx reads from the UDP port
        // and sends packets to relevant parties
        tokio::spawn(async move {
            if let Err(e) = udp_rx.run().await {
                panic!("UdpRx threw error: {}", e)
            }
        });

        // udp_tx writes to the UDP port and maintains
        // gateway to IP map
        tokio::spawn(async move {
            if let Err(e) = udp_tx.run().await {
                panic!("UdpTx threw error: {}", e)
            }
        });

        // translates client requests into UdpTxMessage of private type
        tokio::spawn(async move {
            if let Err(e) = client_rx_translator.run().await {
                panic!("UdpRx threw error: {}", e)
            }
        });

        Ok(UdpRuntime {
            tx: client_tx,
            rx: client_rx,
        })
    }
}

impl ClientRxTranslator {
    pub async fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let msg = self.receiver.recv().await;
            if let Some((packet, mac)) = msg {
                self.udp_tx_sender
                    .send(UdpMessage::PacketByMac((packet, mac)))
                    .await?;
            }
        }
    }
}

impl UdpRx {
    pub async fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut buf = vec![0u8; 1024];
        loop {
            match self.socket_receiver.recv_from(&mut buf).await {
                Err(e) => return Err(e.into()),
                Ok((n, src)) => {
                    let packet = if let Ok(packet) = Packet::parse(&buf[0..n], n) {
                        Some(packet)
                    } else {
                        let mut vec = Vec::new();
                        vec.extend_from_slice(&buf);
                        self.client_tx_sender
                            .send(Event::UnableToParseUdpFrame(vec))
                            .unwrap();
                        None
                    };

                    if let Some(packet) = packet {
                        match packet {
                            Packet::Up(packet) => {
                                // echo all packets to client
                                self.client_tx_sender
                                    .send(Event::Packet(packet.clone()))
                                    .unwrap();

                                match packet {
                                    Up::PullData(pull_data) => {
                                        let mac = pull_data.gateway_mac;
                                        // first send (mac, addr) to update map owned by UdpRuntimeTx
                                        let client = (mac, src);
                                        self.udp_tx_sender.send(UdpMessage::Client(client)).await?;

                                        // send the ack_packet
                                        let ack_packet = pull_data.into_ack();
                                        let mut udp_tx = self.udp_tx_sender.clone();
                                        udp_tx
                                            .send(UdpMessage::PacketByMac((ack_packet.into(), mac)))
                                            .await
                                            .unwrap()
                                    }
                                    Up::PushData(push_data) => {
                                        let socket_addr = src;
                                        // send the ack_packet
                                        let ack_packet = push_data.into_ack();
                                        self.udp_tx_sender
                                            .send(UdpMessage::PacketBySocket((
                                                ack_packet.into(),
                                                socket_addr,
                                            )))
                                            .await?;
                                    }
                                    _ => (),
                                }
                            }
                            Packet::Down(_) => {
                                panic!("Should not receive this frame from forwarder")
                            }
                        };
                    }
                }
            }
        }
    }
}

impl UdpTx {
    pub async fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut buf = vec![0u8; 1024];
        loop {
            let msg = self.receiver.recv().await;
            if let Some(msg) = msg {
                match msg {
                    UdpMessage::PacketByMac((packet, mac)) => {
                        if let Some(addr) = self.clients.get(&mac) {
                            let n = packet.serialize(&mut buf)? as usize;
                            let _sent = self.socket_sender.send_to(&buf[..n], addr).await?;
                        } else {
                            if let Packet::Down(Down::PullResp(pull_resp)) = packet {
                                self.client_tx_sender
                                    .send(Event::Packet(Up::TxAck(
                                        pull_resp.into_nack_for_client(mac),
                                    )))
                                    .unwrap();
                            }
                        }
                    }
                    UdpMessage::PacketBySocket((packet, addr)) => {
                        let n = packet.serialize(&mut buf)? as usize;
                        let _sent = self.socket_sender.send_to(&buf[..n], &addr).await?;
                    }
                    UdpMessage::Client((mac, addr)) => {
                        // tell user if same MAC has new IP
                        if let Some(existing_addr) = self.clients.get(&mac) {
                            if *existing_addr != addr {
                                self.clients.insert(mac, addr);
                                self.client_tx_sender
                                    .send(Event::UpdateClient((mac, addr)))
                                    .unwrap();
                            }
                        }
                        // simply insert if no entry exists
                        else {
                            self.clients.insert(mac, addr);
                            self.client_tx_sender
                                .send(Event::NewClient((mac, addr)))
                                .unwrap();
                        }
                    }
                }
            }
        }
    }
}