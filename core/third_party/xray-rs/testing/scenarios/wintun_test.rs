#[cfg(test)]
mod tests {
    use crate::proxy::tun::config::TunConfig;
    use crate::proxy::tun::handler::TunHandler;
    use crate::proxy::tun::udp_fullcone::UdpNatTable;
    use std::net::Ipv4Addr;
    use std::time::Duration;

    #[test]
    fn test_tun_ipv4_tcp_packet_parsing() {
        // Construct standard IPv4 TCP packet
        let mut packet = vec![0u8; 40];
        packet[0] = 0x45; // IPv4, IHL = 5 (20 bytes)
        packet[9] = 6; // TCP protocol
        // Src IP: 192.168.1.100
        packet[12] = 192;
        packet[13] = 168;
        packet[14] = 1;
        packet[15] = 100;
        // Dst IP: 8.8.8.8
        packet[16] = 8;
        packet[17] = 8;
        packet[18] = 8;
        packet[19] = 8;
        // Src Port: 54321 (0xD431)
        packet[20] = 0xD4;
        packet[21] = 0x31;
        // Dst Port: 443 (0x01BB)
        packet[22] = 0x01;
        packet[23] = 0xBB;

        let (src, dst, proto, net_type, src_port, dst_port) =
            TunHandler::parse_ip_packet(&packet).unwrap();
        assert_eq!(src, Ipv4Addr::new(192, 168, 1, 100));
        assert_eq!(dst, Ipv4Addr::new(8, 8, 8, 8));
        assert_eq!(proto, 6);
        assert_eq!(net_type, "tcp");
        assert_eq!(src_port, 54321);
        assert_eq!(dst_port, 443);
    }

    #[test]
    fn test_tun_ipv4_udp_packet_parsing() {
        // Construct standard IPv4 UDP packet
        let mut packet = vec![0u8; 28];
        packet[0] = 0x45; // IPv4, IHL = 5
        packet[9] = 17; // UDP protocol
        // Src IP: 10.0.0.2
        packet[12] = 10;
        packet[13] = 0;
        packet[14] = 0;
        packet[15] = 2;
        // Dst IP: 1.1.1.1
        packet[16] = 1;
        packet[17] = 1;
        packet[18] = 1;
        packet[19] = 1;
        // Src Port: 12345 (0x3039)
        packet[20] = 0x30;
        packet[21] = 0x39;
        // Dst Port: 53 (0x0035 - DNS)
        packet[22] = 0x00;
        packet[23] = 0x35;

        let (src, dst, proto, net_type, src_port, dst_port) =
            TunHandler::parse_ip_packet(&packet).unwrap();
        assert_eq!(src, Ipv4Addr::new(10, 0, 0, 2));
        assert_eq!(dst, Ipv4Addr::new(1, 1, 1, 1));
        assert_eq!(proto, 17);
        assert_eq!(net_type, "udp");
        assert_eq!(src_port, 12345);
        assert_eq!(dst_port, 53);
    }

    #[test]
    fn test_tun_udp_nat_table() {
        let nat = UdpNatTable::new(Duration::from_millis(50));
        let client_addr = "192.168.1.50:50000".parse().unwrap();
        let target_addr = "8.8.8.8:53".parse().unwrap();

        nat.insert(client_addr, target_addr);
        assert_eq!(nat.lookup(&client_addr), Some(target_addr));

        std::thread::sleep(Duration::from_millis(60));
        assert_eq!(nat.lookup(&client_addr), None);
    }

    #[test]
    fn test_tun_config_defaults() {
        let cfg = TunConfig::default();
        assert_eq!(cfg.name, "tun0");
        assert_eq!(cfg.mtu, 1500);
    }
}
