// Module: transport\internet\finalmask\udp_test.rs
#[cfg(test)]
mod tests {
    use super::super::finalmask::{
        FINALMASK_VERSION, HeaderManager, HeaderMask, UDP_SIZE, UdpmaskManager,
    };
    use super::super::header::custom::config::{UDPConfig, UDPItem};
    use super::super::header::custom::udp::{UdpCustomClient, UdpCustomServer};
    use super::super::header::dns::conn::DnsPacketConn;
    use super::super::header::dtls::conn::DtlsPacketConn;
    use super::super::header::srtp::conn::SrtpPacketConn;
    use super::super::header::utp::conn::UtpPacketConn;
    use super::super::header::wechat::conn::WeChatPacketConn;
    use super::super::header::wireguard::conn::WireguardPacketConn;

    #[test]
    fn test_finalmask_constants() {
        assert_eq!(FINALMASK_VERSION, 1);
        assert_eq!(UDP_SIZE, 4096);
    }

    #[test]
    fn test_dtls_packet_conn() {
        let mut dtls = DtlsPacketConn::new();
        assert_eq!(dtls.size(), 13);
        let payload = b"udp-dtls-payload";
        let wrapped = dtls.wrap_payload(payload);
        assert_eq!(wrapped.len(), 13 + payload.len());
        assert_eq!(wrapped[0], 23); // ContentType
        let unwrapped = dtls.unwrap_payload(&wrapped).expect("unwrap success");
        assert_eq!(unwrapped, payload);
    }

    #[test]
    fn test_wechat_packet_conn() {
        let mut wechat = WeChatPacketConn::new();
        assert_eq!(wechat.size(), 13);
        let payload = b"udp-wechat-video";
        let wrapped = wechat.wrap_payload(payload);
        assert_eq!(wrapped.len(), 13 + payload.len());
        assert_eq!(wrapped[0], 0xa1);
        assert_eq!(wrapped[1], 0x08);
        let unwrapped = wechat.unwrap_payload(&wrapped).expect("unwrap success");
        assert_eq!(unwrapped, payload);
    }

    #[test]
    fn test_dns_packet_conn() {
        let dns = DnsPacketConn::new("www.cloudflare.com").expect("create dns conn");
        assert!(dns.size() > 12);
        let payload = b"dns-tunnel-data";
        let wrapped = dns.wrap_payload(payload);
        assert_eq!(wrapped.len(), dns.size() + payload.len());
        let unwrapped = dns.unwrap_payload(&wrapped).expect("unwrap success");
        assert_eq!(unwrapped, payload);
    }

    #[test]
    fn test_srtp_utp_wireguard_conns() {
        let mut srtp = SrtpPacketConn::new();
        assert_eq!(srtp.size(), 4);
        let payload = b"srtp-audio";
        let wrapped = srtp.wrap_payload(payload);
        assert_eq!(srtp.unwrap_payload(&wrapped).unwrap(), payload);

        let utp = UtpPacketConn::new();
        assert_eq!(utp.size(), 4);
        let wrapped_utp = utp.wrap_payload(payload);
        assert_eq!(utp.unwrap_payload(&wrapped_utp).unwrap(), payload);

        let wg = WireguardPacketConn::new();
        assert_eq!(wg.size(), 4);
        let wrapped_wg = wg.wrap_payload(payload);
        assert_eq!(wg.unwrap_payload(&wrapped_wg).unwrap(), payload);
    }

    #[test]
    fn test_custom_udp_client_server() {
        let cfg = UDPConfig {
            client: vec![
                UDPItem {
                    rand: 0,
                    rand_min: 0,
                    rand_max: 0,
                    packet: b"MAGIC-REQ".to_vec(),
                },
                UDPItem {
                    rand: 4,
                    rand_min: 0xaa,
                    rand_max: 0xaa,
                    packet: vec![],
                },
            ],
            server: vec![UDPItem {
                rand: 0,
                rand_min: 0,
                rand_max: 0,
                packet: b"MAGIC-RESP".to_vec(),
            }],
        };

        let client = UdpCustomClient::new(&cfg);
        let server = UdpCustomServer::new(&cfg);

        // Client sends packet to server
        let client_payload = b"client-to-server-udp";
        let client_packet = client.wrap_payload(client_payload);
        assert!(server.matches(&client_packet));
        let server_received = server.unwrap_payload(&client_packet).unwrap();
        assert_eq!(server_received, client_payload);

        // Server replies to client
        let server_payload = b"server-to-client-udp";
        let server_packet = server.wrap_payload(server_payload);
        assert!(client.matches(&server_packet));
        let client_received = client.unwrap_payload(&server_packet).unwrap();
        assert_eq!(client_received, server_payload);
    }

    #[test]
    fn test_header_manager_stacking() {
        let headers: Vec<Box<dyn HeaderMask>> = vec![
            Box::new(DtlsPacketConn::new()),
            Box::new(WeChatPacketConn::new()),
            Box::new(SrtpPacketConn::new()),
        ];

        let mut mgr = HeaderManager::new(headers);
        assert_eq!(mgr.total_size(), 13 + 13 + 4); // 30 bytes
        assert_eq!(mgr.len(), 3);
        assert!(!mgr.is_empty());

        let payload = b"deep-stacked-obfuscated-payload";
        let wrapped = mgr.wrap_outgoing(payload).expect("wrap success");
        assert_eq!(wrapped.len(), 30 + payload.len());

        let unwrapped = mgr.unwrap_incoming(&wrapped).expect("unwrap success");
        assert_eq!(unwrapped, payload);

        // Test short packet rejection
        let short_packet = &wrapped[..20];
        assert!(mgr.unwrap_incoming(short_packet).is_err());
    }

    #[test]
    fn test_udpmask_manager() {
        let headers: Vec<Box<dyn HeaderMask>> = vec![Box::new(SrtpPacketConn::new())];
        let header_mgr = HeaderManager::new(headers);
        let mut udp_mgr = UdpmaskManager::new(Some(header_mgr), vec![]);

        let payload = b"udpmask-manager-data";
        let wrapped = udp_mgr.wrap_client(payload).unwrap();
        assert_eq!(wrapped.len(), 4 + payload.len());

        let unwrapped = udp_mgr.unwrap_client(&wrapped).unwrap();
        assert_eq!(unwrapped, payload);
    }
}
