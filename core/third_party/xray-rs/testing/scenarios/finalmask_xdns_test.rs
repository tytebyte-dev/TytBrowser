// Module: testing\scenarios\finalmask_xdns_test.rs
#[cfg(test)]
mod tests {
    use crate::transport::internet::finalmask::xdns::client::XDnsClient;
    use crate::transport::internet::finalmask::xdns::config::XDnsConfig;
    use crate::transport::internet::finalmask::xdns::server::XDnsServer;

    #[test]
    fn test_finalmask_xdns_flow() {
        let config = XDnsConfig {
            domain: "fake.lan".into(),
            fake_ip: "10.0.0.1".into(),
        };
        let client = XDnsClient::from_config(&config).expect("Client should create");
        let server = XDnsServer::from_config(&config).expect("Server should create");

        let payload = b"hello from scenario test";
        let query_wire = client.encode_packet(Some(payload)).expect("Encode query");
        let (query_info, mut resp_msg) = server.decode_query(&query_wire).expect("Decode query");

        assert_eq!(query_info.packets.len(), 1);
        assert_eq!(query_info.packets[0], payload);

        let resp_wire = server
            .encode_response(&mut resp_msg, &[b"scenario reply"])
            .expect("Encode reply");
        let client_packets = client.decode_packet(&resp_wire).expect("Decode reply");
        assert_eq!(client_packets.len(), 1);
        assert_eq!(client_packets[0], b"scenario reply");
    }
}
