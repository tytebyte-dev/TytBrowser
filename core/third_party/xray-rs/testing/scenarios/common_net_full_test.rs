#[cfg(test)]
mod tests {
    use crate::common::net::cnc::CncConnection;
    use crate::common::net::{Address, Destination, Network, Port, PortList, PortRange};
    use std::net::{Ipv4Addr, SocketAddr};
    use std::str::FromStr;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[test]
    fn test_port_and_port_range_and_list() {
        let p80 = Port::new(80);
        let p443 = Port::from_str("443").unwrap();
        let p_bytes = Port::from_bytes(&[0x1F, 0x90]).unwrap(); // 8080

        assert_eq!(p80.value(), 80);
        assert_eq!(p443.value(), 443);
        assert_eq!(p_bytes.value(), 8080);
        assert_eq!(p80.to_string(), "80");

        let range = PortRange::new(80, 443);
        assert!(range.contains(Port::new(80)));
        assert!(range.contains(Port::new(443)));
        assert!(range.contains(Port::new(100)));
        assert!(!range.contains(Port::new(79)));
        assert!(!range.contains(Port::new(444)));

        let mut list = PortList::new();
        list.add_single(22);
        list.add_range(8000, 9000);

        assert!(list.contains(Port::new(22)));
        assert!(list.contains(Port::new(8080)));
        assert!(!list.contains(Port::new(23)));
        assert!(!list.contains(Port::new(9001)));
    }

    #[test]
    fn test_address_and_destination() {
        let addr_ip = Address::from_str("1.1.1.1").unwrap();
        assert!(addr_ip.is_ip());
        assert!(!addr_ip.is_domain());
        assert_eq!(addr_ip.to_string(), "1.1.1.1");

        let addr_domain = Address::from_str("example.com").unwrap();
        assert!(!addr_domain.is_ip());
        assert!(addr_domain.is_domain());
        assert_eq!(addr_domain.to_string(), "example.com");

        let dest = Destination::tcp(addr_domain, 443);
        assert_eq!(dest.network, Network::Tcp);
        assert_eq!(dest.port, 443);
        assert_eq!(dest.to_string(), "example.com:443");

        let dest_udp =
            Destination::from_ip_port(std::net::IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), 53);
        assert_eq!(dest_udp.port, 53);
    }

    #[tokio::test]
    async fn test_cnc_connection_duplex() {
        let (client_r, client_w) = tokio::io::duplex(64);
        let (server_r, _server_w) = tokio::io::duplex(64);

        let local: SocketAddr = "127.0.0.1:12345".parse().unwrap();
        let remote: SocketAddr = "127.0.0.1:54321".parse().unwrap();

        let mut conn = CncConnection::new(server_r, client_w, Some(local), Some(remote));
        assert_eq!(conn.local_addr(), Some(local));
        assert_eq!(conn.remote_addr(), Some(remote));

        conn.write_all(b"ping").await.unwrap();
        let mut buf = [0u8; 4];
        let mut client_r_pinned = client_r;
        client_r_pinned.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"ping");
    }
}
