// Module: transport\internet\kcp\kcp_test.rs
// 1:1 Rust unit test suite corresponding to Go transport\internet\kcp\kcp_test.go

#[cfg(test)]
mod tests {
    use super::super::receiving::ReceivingWindow;
    use super::super::segment::DataSegment;
    use super::super::sending::SendingWindow;

    #[test]
    fn test_kcp_windows_sliding() {
        let mut send_win = SendingWindow::new();
        send_win.push(DataSegment::new(1, 0, b"a".to_vec()));
        send_win.push(DataSegment::new(1, 1, b"b".to_vec()));
        assert_eq!(send_win.len(), 2);

        let rtt = send_win.acknowledge(0, 100);
        assert!(rtt.is_some());
        assert_eq!(send_win.len(), 1);

        let mut recv_win = ReceivingWindow::new();
        assert!(recv_win.set(DataSegment::new(1, 0, b"a".to_vec())));
        assert!(!recv_win.set(DataSegment::new(1, 0, b"a".to_vec()))); // duplicate rejected

        let mut stream = Vec::new();
        let drained = recv_win.drain_in_order(&mut stream);
        assert_eq!(drained, 1);
        assert_eq!(stream, b"a");
    }

    #[test]
    fn test_kcp_config_in_flight_sizes() {
        use super::super::config::KcpConfig;
        let mut cfg = KcpConfig::default();
        cfg.uplink_capacity = 10;
        cfg.downlink_capacity = 40;
        cfg.mtu = 1400;
        cfg.tti = 50;
        assert!(cfg.sending_in_flight_size() >= 8);
        assert!(cfg.receiving_in_flight_size() >= 8);
        assert!(cfg.receiving_in_flight_size() > cfg.sending_in_flight_size());
    }
}
