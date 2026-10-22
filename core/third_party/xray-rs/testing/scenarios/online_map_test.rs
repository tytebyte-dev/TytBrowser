#[cfg(test)]
mod tests {
    use crate::app::stats::OnlineMap;

    #[test]
    fn test_online_map_add_remove_and_count() {
        let om = OnlineMap::new();

        om.add_ip("192.168.1.100");
        om.add_ip("192.168.1.100");
        om.add_ip("10.0.0.1");

        // Localhost should be ignored
        om.add_ip("127.0.0.1");
        om.add_ip("::1");

        assert_eq!(om.count(), 2);

        let ips = om.list_ips();
        assert!(ips.contains(&"192.168.1.100".to_string()));
        assert!(ips.contains(&"10.0.0.1".to_string()));

        om.remove_ip("192.168.1.100");
        assert_eq!(om.count(), 2); // ref_count was 2, now 1

        om.remove_ip("192.168.1.100");
        assert_eq!(om.count(), 1); // now removed

        om.remove_ip("10.0.0.1");
        assert_eq!(om.count(), 0);
    }
}
