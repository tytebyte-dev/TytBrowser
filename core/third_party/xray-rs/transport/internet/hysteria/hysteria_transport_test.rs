// Module: transport\internet\hysteria\hysteria_transport_test.rs
// 1:1 Rust unit test suite corresponding to Go transport\internet\hysteria

#[cfg(test)]
mod tests {
    use super::super::config::HysteriaTransportConfig;
    use super::super::congestion::bbr::bandwidth::Bandwidth;
    use super::super::congestion::bbr::packet_number_indexed_queue::PacketNumberIndexedQueue;
    use super::super::congestion::bbr::ringbuffer::RingBuffer;
    use super::super::congestion::bbr::windowed_filter::WindowedMaxFilter;
    use super::super::congestion::brutal::brutal::BrutalSender;
    use super::super::congestion::common::pacer::Pacer;
    use super::super::conn::HysteriaConn;
    use super::super::dialer::HysteriaDialer;
    use super::super::hub::HysteriaHub;
    use super::super::padding::padding::{PADDING_CHARS, Padding};
    use super::super::udphop::addr::UDPHopAddr;
    use super::super::udphop::conn::UdpHopPacketConn;
    use std::net::{IpAddr, Ipv4Addr};
    use std::time::{Duration, Instant};

    #[test]
    fn test_hysteria_padding_generation() {
        let p = Padding::new(10, 20);
        let s = p.generate_string();
        assert!(s.len() >= 10 && s.len() < 20);
        for c in s.chars() {
            assert!(PADDING_CHARS.contains(&(c as u8)));
        }
    }

    #[test]
    fn test_udphop_addr_and_conn_cycle() {
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let hop_addr = UDPHopAddr::new(ip, vec![10001, 10002, 10003], "10001-10003");
        assert_eq!(hop_addr.network(), "udphop");
        assert_eq!(hop_addr.addrs().len(), 3);

        // Interval < 5 seconds rejected
        let bad = UdpHopPacketConn::new(
            hop_addr.clone(),
            Duration::from_secs(2),
            Duration::from_secs(3),
        );
        assert!(bad.is_err());

        // Valid conn
        let conn =
            UdpHopPacketConn::new(hop_addr, Duration::from_secs(10), Duration::from_secs(20))
                .expect("valid hop conn");
        assert_eq!(conn.current_addr().port(), 10001);
        assert_eq!(conn.hop().port(), 10002);
        assert_eq!(conn.hop().port(), 10003);
        assert_eq!(conn.hop().port(), 10001); // loops back
    }

    #[test]
    fn test_pacer_and_brutal_sender() {
        let mut pacer = Pacer::new(8_000_000); // 8 Mbps = 1 MB/s
        let now = Instant::now();
        assert_eq!(pacer.budget(now), 12000); // 10 * 1200
        pacer.sent_packet(now, 5000);
        assert_eq!(pacer.budget(now), 7000);

        let mut brutal = BrutalSender::new(10_000_000);
        assert_eq!(brutal.pacing_rate(), 10_000_000);
        // Record 80 acks and 20 losses at t=100
        brutal.record_ack(100, 80);
        brutal.record_loss(100, 20);
        assert!((brutal.ack_rate - 0.8).abs() < 1e-4);
        assert_eq!(brutal.pacing_rate(), 12_500_000);
    }

    #[test]
    fn test_bbr_data_structures() {
        // Bandwidth
        let bw = Bandwidth::from_bytes_and_duration(1_000_000, Duration::from_secs(1));
        assert_eq!(bw.bps(), 8_000_000);
        assert_eq!(bw.bytes_per_sec(), 1_000_000);

        // WindowedMaxFilter
        let mut filter = WindowedMaxFilter::new(Duration::from_secs(5));
        let t0 = Instant::now();
        filter.update(100, t0);
        assert_eq!(filter.get_best(), 100);
        filter.update(200, t0);
        assert_eq!(filter.get_best(), 200);

        // RingBuffer
        let mut rb = RingBuffer::new(3);
        rb.push(1);
        rb.push(2);
        rb.push(3);
        rb.push(4);
        assert_eq!(rb.len(), 3);

        // PacketNumberIndexedQueue
        let mut queue = PacketNumberIndexedQueue::new();
        queue.insert(1, "pkt1");
        queue.insert(2, "pkt2");
        assert_eq!(queue.remove(1), Some("pkt1"));
        assert_eq!(queue.len(), 1);
    }

    #[tokio::test]
    async fn test_hysteria_conn_tracking_and_hub() {
        let conn = HysteriaConn::new();
        conn.record_sent(1024);
        conn.record_recv(2048);
        assert_eq!(conn.total_sent(), 1024);
        assert_eq!(conn.total_recv(), 2048);

        let dialer = HysteriaDialer::new(HysteriaTransportConfig::default());
        let addr = "127.0.0.1:443".parse().unwrap();
        let dialed = dialer.dial(addr).await;
        assert_eq!(dialed.total_sent(), 0);

        let hub = HysteriaHub::new();
        hub.register(1, "tag1".to_string());
        hub.remove(1);
    }
}
