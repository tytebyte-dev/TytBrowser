// Module: app\policy\manager_test.rs
// 1:1 Rust unit test suite corresponding to Go app\policy\manager_test.go

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::time::Duration;

    use crate::app::policy::PolicyLevelConfig;
    use crate::app::policy::config_pb::{Config, Policy, PolicyTimeout, Second};
    use crate::app::policy::manager::{ConfigPolicyManager, Instance};
    use crate::features::policy::session_default;

    #[test]
    fn test_policy_manager_instance_overrides() {
        let mut level_map = HashMap::new();
        level_map.insert(
            0,
            Policy {
                timeout: Some(PolicyTimeout {
                    handshake: Some(Second::new(2)),
                    ..Default::default()
                }),
                ..Default::default()
            },
        );

        let cfg = Config {
            level: level_map,
            system: None,
        };

        let manager = Instance::new(&cfg).expect("create policy instance");
        let p_default = session_default();

        // Level 0 has overridden handshake = 2s, but inherits connection_idle default
        let p0 = manager.for_level(0);
        assert_eq!(p0.timeouts.handshake, Duration::from_secs(2));
        assert_eq!(
            p0.timeouts.connection_idle,
            p_default.timeouts.connection_idle
        );

        // Level 1 was not explicitly defined, so falls back to session_default()
        let p1 = manager.for_level(1);
        assert_eq!(p1.timeouts.handshake, p_default.timeouts.handshake);
        assert_eq!(
            p1.timeouts.connection_idle,
            p_default.timeouts.connection_idle
        );
    }

    #[test]
    fn test_policy_manager_levels() {
        let pm = ConfigPolicyManager::new();
        let mut p = PolicyLevelConfig::default();
        p.handshake_timeout = Some(10);
        p.stats_user_uplink = true;

        pm.set_level_policy(1, p);
        let ret = pm.for_level(1);
        assert_eq!(ret.handshake_timeout, Some(10));
        assert!(ret.stats_user_uplink);

        let ret0 = pm.for_level(0);
        assert_eq!(ret0.handshake_timeout, None);
    }
}
