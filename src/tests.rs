use super::packet::parser::Parser;
use super::*;
#[test]
fn test_pull_data() {
    let recv = [
        0x2, 0x9F, 0x92, 0x2, 0xAA, 0x55, 0x5A, 0x1, 0x2, 0x3, 0x4, 0x5,
    ];
    let packet = Packet::parse(&recv, recv.len()).unwrap();

    if let Packet::Up(Up::PullData(packet)) = packet {
        let mut buffer = [0; 512];
        let written = packet.serialize(&mut buffer).unwrap();
        assert_eq!(written, recv.len() as u64);

        for i in 0..recv.len() {
            assert_eq!(recv[i], buffer[i]);
        }
    } else {
        assert!(false);
    }
}

#[test]
fn test_push_data_rxpk() {
    let recv = [
        0x2, 0x5E, 0x52, 0x0, 0xAA, 0x55, 0x5A, 0x0, 0x0, 0x0, 0x0, 0x0, 0x7B, 0x22, 0x72, 0x78,
        0x70, 0x6B, 0x22, 0x3A, 0x5B, 0x7B, 0x22, 0x74, 0x6D, 0x73, 0x74, 0x22, 0x3A, 0x31, 0x34,
        0x37, 0x32, 0x32, 0x34, 0x32, 0x32, 0x35, 0x32, 0x2C, 0x22, 0x63, 0x68, 0x61, 0x6E, 0x22,
        0x3A, 0x38, 0x2C, 0x22, 0x72, 0x66, 0x63, 0x68, 0x22, 0x3A, 0x30, 0x2C, 0x22, 0x66, 0x72,
        0x65, 0x71, 0x22, 0x3A, 0x39, 0x31, 0x32, 0x2E, 0x36, 0x30, 0x30, 0x30, 0x30, 0x30, 0x2C,
        0x22, 0x73, 0x74, 0x61, 0x74, 0x22, 0x3A, 0x31, 0x2C, 0x22, 0x6D, 0x6F, 0x64, 0x75, 0x22,
        0x3A, 0x22, 0x4C, 0x4F, 0x52, 0x41, 0x22, 0x2C, 0x22, 0x64, 0x61, 0x74, 0x72, 0x22, 0x3A,
        0x22, 0x53, 0x46, 0x38, 0x42, 0x57, 0x35, 0x30, 0x30, 0x22, 0x2C, 0x22, 0x63, 0x6F, 0x64,
        0x72, 0x22, 0x3A, 0x22, 0x34, 0x2F, 0x35, 0x22, 0x2C, 0x22, 0x6C, 0x73, 0x6E, 0x72, 0x22,
        0x3A, 0x31, 0x30, 0x2E, 0x38, 0x2C, 0x22, 0x72, 0x73, 0x73, 0x69, 0x22, 0x3A, 0x2D, 0x35,
        0x38, 0x2C, 0x22, 0x73, 0x69, 0x7A, 0x65, 0x22, 0x3A, 0x32, 0x33, 0x2C, 0x22, 0x64, 0x61,
        0x74, 0x61, 0x22, 0x3A, 0x22, 0x41, 0x4C, 0x51, 0x41, 0x41, 0x41, 0x41, 0x42, 0x41, 0x41,
        0x41, 0x41, 0x53, 0x47, 0x56, 0x73, 0x61, 0x58, 0x56, 0x74, 0x49, 0x43, 0x41, 0x30, 0x4C,
        0x44, 0x59, 0x43, 0x4E, 0x72, 0x41, 0x3D, 0x22, 0x7D, 0x5D, 0x7D,
    ];

    let packet = Packet::parse(&recv, recv.len()).unwrap();

    if let Packet::Up(Up::PushData(packet)) = packet {
        let mut buffer = [0; 512];
        let written = packet.serialize(&mut buffer).unwrap();
        let _packet = Packet::parse(&buffer, written as usize).unwrap();
    } else {
        assert!(false);
    }
}

#[test]
fn test_push_data_rxpk_jsonv2() {
    let recv = [
        2, 120, 20, 0, 114, 118, 255, 0, 68, 1, 0, 16, 123, 34, 114, 120, 112, 107, 34, 58, 91,
        123, 34, 97, 101, 115, 107, 34, 58, 48, 44, 34, 98, 114, 100, 34, 58, 48, 44, 34, 99, 111,
        100, 114, 34, 58, 34, 52, 47, 53, 34, 44, 34, 100, 97, 116, 97, 34, 58, 34, 81, 65, 65, 65,
        65, 69, 103, 65, 69, 116, 99, 68, 118, 75, 55, 110, 100, 109, 66, 70, 66, 103, 61, 61, 34,
        44, 34, 100, 97, 116, 114, 34, 58, 34, 83, 70, 49, 48, 66, 87, 49, 50, 53, 34, 44, 34, 102,
        114, 101, 113, 34, 58, 57, 48, 51, 46, 57, 44, 34, 106, 118, 101, 114, 34, 58, 50, 44, 34,
        109, 111, 100, 117, 34, 58, 34, 76, 79, 82, 65, 34, 44, 34, 114, 115, 105, 103, 34, 58, 91,
        123, 34, 97, 110, 116, 34, 58, 48, 44, 34, 99, 104, 97, 110, 34, 58, 48, 44, 34, 108, 115,
        110, 114, 34, 58, 49, 48, 46, 48, 44, 34, 114, 115, 115, 105, 99, 34, 58, 45, 52, 54, 125,
        93, 44, 34, 115, 105, 122, 101, 34, 58, 49, 54, 44, 34, 115, 116, 97, 116, 34, 58, 49, 44,
        34, 116, 105, 109, 101, 34, 58, 34, 50, 48, 50, 48, 45, 49, 48, 45, 50, 57, 84, 49, 53, 58,
        53, 55, 58, 52, 48, 46, 49, 55, 48, 51, 48, 49, 90, 34, 44, 34, 116, 109, 115, 116, 34, 58,
        51, 49, 51, 57, 57, 56, 56, 55, 54, 125, 93, 125,
    ];

    let packet = Packet::parse(&recv, recv.len()).unwrap();

    if let Packet::Up(Up::PushData(packet)) = packet {
        let mut buffer = [0; 512];
        let written = packet.serialize(&mut buffer).unwrap();
        let _packet = Packet::parse(&buffer, written as usize).unwrap();
    } else {
        assert!(false);
    }
}

#[test]
fn test_push_data_stat() {
    let recv = [
        0x2, 0x86, 0xBE, 0x0, 0xAA, 0x55, 0x5A, 0x0, 0x0, 0x0, 0x0, 0x0, 0x7B, 0x22, 0x73, 0x74,
        0x61, 0x74, 0x22, 0x3A, 0x7B, 0x22, 0x74, 0x69, 0x6D, 0x65, 0x22, 0x3A, 0x22, 0x32, 0x30,
        0x32, 0x30, 0x2D, 0x30, 0x33, 0x2D, 0x30, 0x34, 0x20, 0x30, 0x37, 0x3A, 0x30, 0x31, 0x3A,
        0x30, 0x32, 0x20, 0x47, 0x4D, 0x54, 0x22, 0x2C, 0x22, 0x72, 0x78, 0x6E, 0x62, 0x22, 0x3A,
        0x33, 0x2C, 0x22, 0x72, 0x78, 0x6F, 0x6B, 0x22, 0x3A, 0x33, 0x2C, 0x22, 0x72, 0x78, 0x66,
        0x77, 0x22, 0x3A, 0x33, 0x2C, 0x22, 0x61, 0x63, 0x6B, 0x72, 0x22, 0x3A, 0x30, 0x2E, 0x30,
        0x2C, 0x22, 0x64, 0x77, 0x6E, 0x62, 0x22, 0x3A, 0x30, 0x2C, 0x22, 0x74, 0x78, 0x6E, 0x62,
        0x22, 0x3A, 0x30, 0x7D, 0x7D,
    ];

    let packet = Packet::parse(&recv, recv.len()).unwrap();

    if let Packet::Up(Up::PushData(packet)) = packet {
        let _packet_first_read = Packet::parse(&recv, recv.len()).unwrap();

        let mut buffer_first = [0; 512];
        let written_first = packet.serialize(&mut buffer_first).unwrap();

        let packet_second_read = Packet::parse(&buffer_first, written_first as usize).unwrap();
        if let Packet::Up(Up::PushData(packet_second_read)) = packet_second_read {
            let mut buffer_second = [0; 512];
            let _written_second = packet_second_read.serialize(&mut buffer_second).unwrap();
        } else {
            assert!(false);
        }
    } else {
        assert!(false);
    }
}

use crate::packet::StringOrNum;

#[test]
fn test_immediate_send() {
    use crate::packet::pull_resp::TxPk;
    let json = "{\"codr\":\"4/5\",\"data\":\"QDDaAAHUbYkmAGY3AFAvfpbHJeCeuDu3xbCCHeg7YPOUJOfBCSc4Y3LtT4aToTGl9AYK4+NiALvTgey0M4ZJzh43vLaaXzFHko0jlb0CVeNgAtbTsAttQ\",\"datr\":\"SF10BW125\",\"freq\":904.1,\"imme\":true,\"ipol\":false,\"modu\":\"LORA\",\"powe\":27,\"rfch\":0,\"size\":87,\"tmst\":\"immediate\"}";

    let txpk: TxPk = serde_json::from_str(json).unwrap();
    if let StringOrNum::S(_) = txpk.tmst {
        assert!(true);
    } else {
        assert!(false);
    }
}
#[test]
fn test_timed_send() {
    use crate::packet::pull_resp::TxPk;
    let json = "{\"codr\":\"4/5\",\"data\":\"IHLF2EA+n8BFY1vrCU1k/Vg=\",\"datr\":\"SF10BW500\",\"freq\":926.9000244140625,\"imme\":false,\"ipol\":true,\"modu\":\"LORA\",\"powe\":27,\"rfch\":0,\"size\":17,\"tmst\":727050748}";

    let txpk: TxPk = serde_json::from_str(json).unwrap();
    if let StringOrNum::N(_) = txpk.tmst {
        assert!(true);
    } else {
        assert!(false);
    }
}
