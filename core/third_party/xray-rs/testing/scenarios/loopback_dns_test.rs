#[cfg(test)]
mod tests {
    use crate::app::dns::DnsClient;
    use crate::common::net::{Address, Destination};
    use crate::proxy::dns::DnsOutbound;
    use std::collections::HashMap;
    use std::net::IpAddr;
    use std::sync::Arc;
    use tokio::io::duplex;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn test_dns_proxy_outbound_response() {
        let mut hosts = HashMap::new();
        hosts.insert(
            "example.com".to_string(),
            vec!["93.184.216.34".parse::<IpAddr>().unwrap()],
        );
        let client = Arc::new(DnsClient::with_hosts(hosts));

        let dns_outbound = DnsOutbound::new("dns_out".into(), client);

        let (mut client_stream, server_stream) = duplex(1024);

        // Craft a basic DNS query for example.com
        let query = vec![
            0x12, 0x34, // Transaction ID
            0x01, 0x00, // Standard query
            0x00, 0x01, // Questions: 1
            0x00, 0x00, // Answer RRs: 0
            0x00, 0x00, // Authority RRs: 0
            0x00, 0x00, // Additional RRs: 0
            // QNAME: 7example3com0
            0x07, b'e', b'x', b'a', b'm', b'p', b'l', b'e', 0x03, b'c', b'o', b'm', 0x00, 0x00,
            0x01, // Type A
            0x00, 0x01, // Class IN
        ];

        let target = Destination::udp(Address::Ipv4("8.8.8.8".parse().unwrap()), 53);

        tokio::spawn(async move {
            dns_outbound.process(server_stream, target).await.unwrap();
        });

        client_stream.write_all(&query).await.unwrap();

        let mut resp = vec![0u8; 512];
        let n = client_stream.read(&mut resp).await.unwrap();
        assert!(n > 12);
        assert_eq!(resp[0], 0x12);
        assert_eq!(resp[1], 0x34);
        // QR flag set in resp[2]
        assert_ne!(resp[2] & 0x80, 0);
        // Answer contains 93.184.216.34 (0x5d, 0xb8, 0xd8, 0x22)
        assert!(resp[..n].windows(4).any(|w| w == [93, 184, 216, 34]));
    }
}
