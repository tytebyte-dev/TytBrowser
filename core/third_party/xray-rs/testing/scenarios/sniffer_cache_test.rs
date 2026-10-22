#[cfg(test)]
mod tests {
    use crate::app::dispatcher::sniffer::Sniffer;
    use crate::app::dispatcher::stats::SizeStatCounter;
    use crate::app::dns::cache_controller::CacheController;
    use crate::app::dns::hosts::StaticHosts;
    use crate::app::stats::Counter;
    use crate::common::net::Network;
    use std::net::IpAddr;
    use std::sync::Arc;
    use std::time::Duration;

    #[test]
    fn test_sniffer_http_host_extraction() {
        let sniffer = Sniffer::new(None);
        let http_payload =
            b"GET / HTTP/1.1\r\nHost: www.example.com:8080\r\nUser-Agent: curl/7.68.0\r\n\r\n";
        let res = sniffer.sniff(http_payload, Network::Tcp).unwrap();
        assert_eq!(res.protocol, "http");
        assert_eq!(res.domain, "www.example.com");
    }

    #[test]
    fn test_sniffer_tls_sni_synthetic() {
        // Construct a synthetic ClientHello with SNI = "xray.com"
        let mut payload = vec![0x16, 0x03, 0x01, 0x00, 0x60]; // TLS Record Header
        payload.push(0x01); // Handshake Type: ClientHello
        payload.extend_from_slice(&[0x00, 0x00, 0x5c]); // Length
        payload.extend_from_slice(&[0x03, 0x03]); // Client Version TLS 1.2
        payload.extend_from_slice(&[0u8; 32]); // Random
        payload.push(0x00); // Session ID Length: 0
        payload.extend_from_slice(&[0x00, 0x02, 0x13, 0x01]); // Cipher Suites
        payload.extend_from_slice(&[0x01, 0x00]); // Compression Methods

        // SNI Extension Data:
        // Type 0 (HostName) + Name Len (2 bytes: 8) + "xray.com" (8 bytes) = 11 bytes
        // Server Name List Length (2 bytes): 11 bytes
        // Total SNI Ext Data = 13 bytes
        // Extension Header: Ext Type (2 bytes: 0x0000) + Ext Len (2 bytes: 13) + Ext Data (13 bytes) = 17 bytes
        payload.extend_from_slice(&[0x00, 0x11]); // Extensions Length (17 bytes)
        payload.extend_from_slice(&[0x00, 0x00]); // Ext Type: SNI (0x0000)
        payload.extend_from_slice(&[0x00, 0x0d]); // Ext Length: 13 bytes
        payload.extend_from_slice(&[0x00, 0x0b]); // Server Name List Length: 11 bytes
        payload.push(0x00); // HostName Type 0
        payload.extend_from_slice(&[0x00, 0x08]); // Length 8
        payload.extend_from_slice(b"xray.com"); // "xray.com"

        let sniffer = Sniffer::new(None);
        let res = sniffer.sniff(&payload, Network::Tcp).unwrap();
        assert_eq!(res.protocol, "tls");
        assert_eq!(res.domain, "xray.com");
    }

    #[test]
    fn test_dns_cache_controller_ttl_and_stale() {
        let cache = CacheController::new("test".into(), false, true, Duration::from_secs(10));
        let ip: IpAddr = "1.2.3.4".parse().unwrap();

        cache.set("foo.com", vec![ip], Duration::from_millis(50));
        assert_eq!(cache.get("foo.com"), Some(vec![ip]));

        // Wait for TTL to expire but still within stale window
        std::thread::sleep(Duration::from_millis(60));
        assert_eq!(cache.get("foo.com"), Some(vec![ip]));
    }

    #[test]
    fn test_static_hosts_domain_suffix_and_keyword() {
        let hosts = StaticHosts::new();
        let ip1: IpAddr = "8.8.8.8".parse().unwrap();
        let ip2: IpAddr = "1.1.1.1".parse().unwrap();

        hosts.add_exact("exact.org", vec![ip1]);
        hosts.add_domain_suffix("google.com", vec![ip2]);

        assert_eq!(hosts.lookup("exact.org"), Some(vec![ip1]));
        assert_eq!(hosts.lookup("mail.google.com"), Some(vec![ip2]));
        assert_eq!(hosts.lookup("google.com"), Some(vec![ip2]));
        assert_eq!(hosts.lookup("notgoogle.org"), None);
    }

    #[test]
    fn test_dispatcher_size_stat_counter() {
        let r = Arc::new(Counter::new());
        let w = Arc::new(Counter::new());
        let stat = SizeStatCounter::new(Some(r.clone()), Some(w.clone()));

        stat.record_read(128);
        stat.record_write(256);

        assert_eq!(r.value(), 128);
        assert_eq!(w.value(), 256);
    }
}
