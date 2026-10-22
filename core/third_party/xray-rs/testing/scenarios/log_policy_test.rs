#[cfg(test)]
mod tests {
    use crate::app::log::{LogLevel, LogManager};
    use crate::app::policy::{PolicyManager, SessionPolicy, SystemPolicy};
    use std::collections::HashMap;
    use std::net::{IpAddr, Ipv4Addr};
    use std::time::Duration;

    #[test]
    fn test_log_manager_level_and_ip_masking() {
        let manager = LogManager::new(LogLevel::Info, 24, 64);
        assert_eq!(manager.get_level(), LogLevel::Info);

        manager.set_level(LogLevel::Debug);
        assert_eq!(manager.get_level(), LogLevel::Debug);

        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 105));
        let masked = manager.mask_ip(&ip);
        assert_eq!(masked, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 0)));

        let log_line =
            manager.format_access_log(Some(&ip), "google.com:443", "accepted", "proxy-out");
        assert!(log_line.contains("192.168.1.0"));
        assert!(log_line.contains("google.com:443"));
        assert!(log_line.contains("[proxy-out]"));
    }

    #[test]
    fn test_policy_manager_levels_and_system() {
        let mut levels = HashMap::new();
        let mut custom_p = SessionPolicy::default();
        custom_p.handshake = Duration::from_secs(10);
        custom_p.conn_idle = Duration::from_secs(600);
        custom_p.buffer_size = 1024 * 1024;
        levels.insert(1, custom_p);

        let sys = SystemPolicy {
            stats_inbound_uplink: true,
            stats_inbound_downlink: true,
            stats_outbound_uplink: true,
            stats_outbound_downlink: true,
        };

        let manager = PolicyManager::new(levels, sys);

        let p0 = manager.for_level(0);
        assert_eq!(p0.handshake, Duration::from_secs(4));

        let p1 = manager.for_level(1);
        assert_eq!(p1.handshake, Duration::from_secs(10));
        assert_eq!(p1.conn_idle, Duration::from_secs(600));
        assert_eq!(p1.buffer_size, 1024 * 1024);

        assert!(manager.for_system().stats_inbound_uplink);
    }
}
