#[cfg(test)]
mod tests {
    use crate::proxy::tun::TunHandler;
    use crate::transport::internet::udp::UdpHub;
    use std::net::SocketAddr;
    use std::sync::atomic::{AtomicU16, Ordering};

    static PORT_COUNTER: AtomicU16 = AtomicU16::new(31500);

    fn pick_port() -> u16 {
        PORT_COUNTER.fetch_add(1, Ordering::SeqCst)
    }

    #[test]
    fn test_tun_ip_packet_parsing() {
        // Construct a synthetic IPv4 UDP packet
        let mut packet = vec![0u8; 28];
        packet[0] = 0x45; // Version 4, IHL 5 (20 bytes)
        packet[9] = 17; // Protocol 17 = UDP
        // Source IP: 192.168.1.100
        packet[12] = 192;
        packet[13] = 168;
        packet[14] = 1;
        packet[15] = 100;
        // Dest IP: 1.1.1.1
        packet[16] = 1;
        packet[17] = 1;
        packet[18] = 1;
        packet[19] = 1;
        // Source Port: 54321 (0xd431)
        packet[20] = 0xd4;
        packet[21] = 0x31;
        // Dest Port: 53 (0x0035)
        packet[22] = 0x00;
        packet[23] = 0x35;

        let (src, dst, proto, net_type, src_port, dst_port) =
            TunHandler::parse_ip_packet(&packet).unwrap();
        assert_eq!(src.to_string(), "192.168.1.100");
        assert_eq!(dst.to_string(), "1.1.1.1");
        assert_eq!(proto, 17);
        assert_eq!(net_type, "udp");
        assert_eq!(src_port, 54321);
        assert_eq!(dst_port, 53);
    }

    #[tokio::test]
    async fn test_udp_hub_send_recv() {
        let port1 = pick_port();
        let port2 = pick_port();

        let addr1: SocketAddr = format!("127.0.0.1:{}", port1).parse().unwrap();
        let addr2: SocketAddr = format!("127.0.0.1:{}", port2).parse().unwrap();

        let mut hub1 = UdpHub::bind(addr1).await.unwrap();
        let hub2 = UdpHub::bind(addr2).await.unwrap();

        hub2.send_to(b"ping from hub2", addr1).await.unwrap();

        let pkt = hub1.recv().await.unwrap();
        assert_eq!(pkt.payload, b"ping from hub2");
        assert_eq!(pkt.source, addr2);
    }
}
