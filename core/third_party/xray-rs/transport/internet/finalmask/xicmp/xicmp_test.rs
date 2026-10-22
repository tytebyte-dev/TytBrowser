// Module: transport\internet\finalmask\xicmp\xicmp_test.rs
// 1:1 Rust unit test suite corresponding to Go transport\internet\finalmask\xicmp\xicmp_test.go

#[cfg(test)]
mod tests {
    use super::super::client::{
        ICMP_TYPE_ECHO_REPLY_V4, ICMP_TYPE_ECHO_REPLY_V6, ICMP_TYPE_ECHO_V4, ICMP_TYPE_ECHO_V6,
        XIcmpClient, calculate_checksum, verify_checksum,
    };
    use super::super::config::XIcmpConfig;
    use super::super::server::XIcmpServer;

    #[test]
    fn test_icmp_echo_marshal_headers() {
        // Corresponds to Go TestICMPEchoMarshal
        // ID: 65535 -> [255, 255], Seq: 65537 -> modulo 65536 -> [0, 1]
        let cfg_v4 = XIcmpConfig {
            ip: "127.0.0.1".into(),
            id: 65535,
            sequence: 1, // 65537 % 65536 = 1
            payload_size: 0,
        };
        let mut client_v4 = XIcmpClient::new(cfg_v4.clone());
        let req_v4 = client_v4.encode_request(None).unwrap();

        assert_eq!(&req_v4[0..2], &[ICMP_TYPE_ECHO_V4, 0]); // [8, 0]
        assert_eq!(&req_v4[4..8], &[255, 255, 0, 1]);
        assert!(verify_checksum(&req_v4));

        let cfg_v6 = XIcmpConfig {
            ip: "::1".into(),
            id: 65535,
            sequence: 1,
            payload_size: 0,
        };
        let mut client_v6 = XIcmpClient::new(cfg_v6.clone());
        let req_v6 = client_v6.encode_request(None).unwrap();

        assert_eq!(&req_v6[0..2], &[ICMP_TYPE_ECHO_V6, 0]); // [128, 0]
        assert_eq!(&req_v6[4..8], &[255, 255, 0, 1]);
        assert!(verify_checksum(&req_v6));

        // Server replies
        let server_v4 = XIcmpServer::new(cfg_v4);
        let req_info_v4 = server_v4.decode_request(&req_v4).unwrap().unwrap();
        let reply_v4 = server_v4.encode_reply(&req_info_v4, None).unwrap();
        assert_eq!(&reply_v4[0..2], &[ICMP_TYPE_ECHO_REPLY_V4, 0]); // [0, 0]
        assert_eq!(&reply_v4[4..8], &[255, 255, 0, 1]);
        assert!(verify_checksum(&reply_v4));

        let server_v6 = XIcmpServer::new(cfg_v6);
        let req_info_v6 = server_v6.decode_request(&req_v6).unwrap().unwrap();
        let reply_v6 = server_v6.encode_reply(&req_info_v6, None).unwrap();
        assert_eq!(&reply_v6[0..2], &[ICMP_TYPE_ECHO_REPLY_V6, 0]); // [129, 0]
        assert_eq!(&reply_v6[4..8], &[255, 255, 0, 1]);
        assert!(verify_checksum(&reply_v6));
    }

    #[test]
    fn test_xicmp_roundtrip_flow_ipv4() {
        let cfg = XIcmpConfig {
            ip: "10.0.0.1".into(),
            id: 0x4321,
            sequence: 10,
            payload_size: 100,
        };
        let mut client = XIcmpClient::new(cfg.clone());
        let server = XIcmpServer::new(cfg);

        let uplink_payload = b"ping data from client over icmp";
        let req_pkt = client
            .encode_request(Some(uplink_payload))
            .expect("Client request");

        let req_info = server
            .decode_request(&req_pkt)
            .expect("Decode request")
            .expect("Some request");
        assert_eq!(req_info.id, 0x4321);
        assert_eq!(req_info.seq, 10);
        assert!(req_info.need_seq_byte);
        assert_eq!(req_info.seq_byte, uplink_payload[0]);
        assert_eq!(req_info.payload, uplink_payload);

        let downlink_payload = b"pong response from server over icmp";
        let reply_pkt = server
            .encode_reply(&req_info, Some(downlink_payload))
            .expect("Server reply");

        let decoded = client
            .decode_reply(&reply_pkt)
            .expect("Decode reply")
            .expect("Some decoded");
        assert_eq!(decoded, downlink_payload);
    }

    #[test]
    fn test_xicmp_roundtrip_flow_ipv6() {
        let cfg = XIcmpConfig {
            ip: "2001:db8::1".into(),
            id: 0x7788,
            sequence: 42,
            payload_size: 128,
        };
        let mut client = XIcmpClient::new(cfg.clone());
        let server = XIcmpServer::new(cfg);

        let payload = b"ipv6 icmp payload";
        let req_pkt = client.encode_request(Some(payload)).unwrap();
        let req_info = server.decode_request(&req_pkt).unwrap().unwrap();
        assert_eq!(req_info.id, 0x7788);
        assert_eq!(req_info.seq, 42);

        let reply_payload = b"ipv6 icmp echo reply payload";
        let reply_pkt = server.encode_reply(&req_info, Some(reply_payload)).unwrap();
        let decoded = client.decode_reply(&reply_pkt).unwrap().unwrap();
        assert_eq!(decoded, reply_payload);
    }

    #[test]
    fn test_xicmp_reflection_rejection() {
        let cfg = XIcmpConfig {
            ip: "10.0.0.1".into(),
            id: 0x1111,
            sequence: 5,
            payload_size: 0,
        };
        let mut client = XIcmpClient::new(cfg);

        let payload = b"protect against echo reflection";
        let _ = client.encode_request(Some(payload)).unwrap();

        // Craft a malicious reflection packet that repeats the client's seq_byte without modification
        let mut reflected = Vec::new();
        reflected.push(ICMP_TYPE_ECHO_REPLY_V4);
        reflected.push(0);
        reflected.extend_from_slice(&[0, 0]); // Checksum placeholder
        reflected.extend_from_slice(&0x1111u16.to_be_bytes());
        reflected.extend_from_slice(&5u16.to_be_bytes());
        // Reflection: first byte matches seq_byte payload[0]
        reflected.push(payload[0]);
        reflected.extend_from_slice(b"extra reflected bytes");

        let csum = calculate_checksum(&reflected);
        reflected[2..4].copy_from_slice(&csum.to_be_bytes());

        // Client must reject reflected packets where data[0] == status.seq_byte
        let result = client.decode_reply(&reflected).unwrap();
        assert_eq!(result, None);
    }
}
