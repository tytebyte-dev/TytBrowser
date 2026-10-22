// Module: app\stats\stats_test.rs
// 1:1 Rust unit test suite corresponding to Go app\stats\stats_test.go

#[cfg(test)]
mod tests {
    use super::super::StatsManager;
    use super::super::online_map::OnlineMap;
    use super::super::stats::*;

    #[test]
    fn test_stats_metric_names() {
        assert_eq!(STATS_INBOUND_UPLINK, "inbound>>>uplink");
        assert_eq!(STATS_INBOUND_DOWNLINK, "inbound>>>downlink");
        assert_eq!(STATS_OUTBOUND_UPLINK, "outbound>>>uplink");
        assert_eq!(STATS_OUTBOUND_DOWNLINK, "outbound>>>downlink");

        assert_eq!(
            inbound_uplink_name("http_in"),
            "inbound>>>http_in>>>traffic>>>uplink"
        );
        assert_eq!(
            outbound_downlink_name("proxy_out"),
            "outbound>>>proxy_out>>>traffic>>>downlink"
        );
        assert_eq!(
            user_uplink_name("user@test.com"),
            "user>>>user@test.com>>>traffic>>>uplink"
        );
    }

    #[test]
    fn test_stats_manager_counters_and_channels() {
        let mgr = StatsManager::new();
        let counter = mgr.register_counter_sync("test_metric");
        counter.add(100);
        assert_eq!(counter.value(), 100);

        let retrieved = mgr.get_counter("test_metric");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().value(), 100);

        let chan = mgr.register_channel_sync("test_chan", 16, 5);
        assert_eq!(chan.subscriber_count(), 0);
        assert!(mgr.get_channel("test_chan").is_some());

        assert!(mgr.unregister_counter("test_metric"));
        assert!(mgr.get_counter("test_metric").is_none());

        assert!(mgr.unregister_channel("test_chan"));
        assert!(mgr.get_channel("test_chan").is_none());
    }

    #[test]
    fn test_online_map_lifecycle() {
        let map = OnlineMap::new();
        assert_eq!(map.count(), 0);

        // Localhost IPs should be ignored
        map.add_ip("127.0.0.1");
        map.add_ip("::1");
        assert_eq!(map.count(), 0);

        // Real IP tracking
        map.add_ip("192.168.1.100");
        assert_eq!(map.count(), 1);
        map.add_ip("192.168.1.100"); // Ref count 2
        assert_eq!(map.count(), 1);

        map.add_ip("10.0.0.1");
        assert_eq!(map.count(), 2);

        let list = map.list();
        assert_eq!(list.len(), 2);
        assert!(list.contains(&"192.168.1.100".to_string()));

        map.remove_ip("192.168.1.100"); // Ref count down to 1
        assert_eq!(map.count(), 2);

        map.remove_ip("192.168.1.100"); // Removed
        assert_eq!(map.count(), 1);

        let times = map.ip_time_map();
        assert_eq!(times.len(), 1);
        assert!(times.contains_key("10.0.0.1"));
    }
}
