// Module: transport\internet\finalmask\xdns\xdns_test.rs
// 1:1 Rust unit test suite for XDnsClient and XDnsServer end-to-end operation

#[cfg(test)]
mod tests {
    use super::super::client::XDnsClient;
    use super::super::dns::*;
    use super::super::server::XDnsServer;

    #[test]
    fn test_xdns_client_encode_and_server_decode_payload() {
        let domain = "xdns.example.com";
        let client_id = [1, 2, 3, 4, 5, 6, 7, 8];
        let client = XDnsClient::new_with_id(domain, client_id).unwrap();
        let server = XDnsServer::new(domain).unwrap();

        let payload = b"ping data from client to server";
        let query_wire = client
            .encode_packet(Some(payload))
            .expect("client encode should succeed");

        let (query_info, resp_skeleton) = server
            .decode_query(&query_wire)
            .expect("server decode should succeed");
        assert_eq!(query_info.client_id, client_id);
        assert_eq!(query_info.rcode, RCODE_NO_ERROR);
        assert_eq!(query_info.packets.len(), 1);
        assert_eq!(query_info.packets[0], payload);
        assert_eq!(resp_skeleton.rcode(), RCODE_NO_ERROR);
    }

    #[test]
    fn test_xdns_client_poll_query() {
        let domain = "poll.test.local";
        let client_id = [0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x11, 0x22];
        let client = XDnsClient::new_with_id(domain, client_id).unwrap();
        let server = XDnsServer::new(domain).unwrap();

        // Encode poll query (no payload)
        let poll_wire = client
            .encode_packet(None)
            .expect("client poll encode should succeed");

        let (query_info, _) = server
            .decode_query(&poll_wire)
            .expect("server decode poll should succeed");
        assert_eq!(query_info.client_id, client_id);
        assert_eq!(query_info.rcode, RCODE_NO_ERROR);
        assert_eq!(query_info.packets.len(), 0); // No data packets in a poll
    }

    #[test]
    fn test_xdns_server_encode_response_and_client_decode() {
        let domain = "resp.example.com";
        let client_id = [9, 8, 7, 6, 5, 4, 3, 2];
        let client = XDnsClient::new_with_id(domain, client_id).unwrap();
        let server = XDnsServer::new(domain).unwrap();

        let req_wire = client.encode_packet(Some(b"request")).unwrap();
        let (_, mut resp_msg) = server.decode_query(&req_wire).unwrap();

        // Server replies with two downlink packets
        let reply_pkt1 = b"downlink packet 1";
        let reply_pkt2 = b"downlink packet 2 - larger binary sequence \x00\x01\x02";
        let resp_wire = server
            .encode_response(&mut resp_msg, &[reply_pkt1, reply_pkt2])
            .expect("server encode response should succeed");

        let decoded_packets = client
            .decode_packet(&resp_wire)
            .expect("client decode response should succeed");
        assert_eq!(decoded_packets.len(), 2);
        assert_eq!(decoded_packets[0], reply_pkt1);
        assert_eq!(decoded_packets[1], reply_pkt2);
    }

    #[test]
    fn test_xdns_domain_mismatch_error() {
        let client = XDnsClient::new("client.domain.com").unwrap();
        let server = XDnsServer::new("different.domain.com").unwrap();

        let req_wire = client.encode_packet(Some(b"mismatch")).unwrap();
        let (query_info, resp) = server.decode_query(&req_wire).unwrap();
        assert_eq!(query_info.rcode, RCODE_NAME_ERROR);
        assert_eq!(resp.rcode(), RCODE_NAME_ERROR);
    }
}
