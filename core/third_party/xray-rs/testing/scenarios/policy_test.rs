// Module: testing\scenarios\policy_test.rs
// Test policy manager and session levels

#[cfg(test)]
mod tests {
    use crate::app::policy::{PolicyManager, SessionPolicy, SystemPolicy};
    use std::collections::HashMap;
    use std::time::Duration;

    #[test]
    fn test_scenario_policy_manager_levels() {
        let mut levels = HashMap::new();
        let custom_policy = SessionPolicy {
            handshake: Duration::from_secs(10),
            conn_idle: Duration::from_secs(60),
            uplink_only: Duration::from_secs(1),
            downlink_only: Duration::from_secs(2),
            buffer_size: 1024 * 1024,
            stats_user_uplink: true,
            stats_user_downlink: true,
        };
        levels.insert(1, custom_policy.clone());

        let system = SystemPolicy {
            stats_inbound_uplink: true,
            stats_inbound_downlink: true,
            stats_outbound_uplink: false,
            stats_outbound_downlink: false,
        };

        let mgr = PolicyManager::new(levels, system);

        // Level 1: custom policy
        let p1 = mgr.for_level(1);
        assert_eq!(p1.handshake, Duration::from_secs(10));
        assert_eq!(p1.buffer_size, 1024 * 1024);
        assert!(p1.stats_user_uplink);

        // Level 0: default fallback policy
        let p0 = mgr.for_level(0);
        assert_eq!(p0.handshake, Duration::from_secs(4));
        assert_eq!(p0.buffer_size, 512 * 1024);
        assert!(!p0.stats_user_uplink);

        // System policy
        assert!(mgr.for_system().stats_inbound_uplink);
        assert!(!mgr.for_system().stats_outbound_uplink);
    }
}
