#[cfg(test)]
mod tests {
    use crate::common::cmdarg::Arg;
    use crate::common::drain::BehaviorSeedLimitedDrainer;
    use crate::common::platform::{EnvFlag, get_asset_location, get_configuration_path};
    use std::io::Cursor;

    #[test]
    fn test_platform_env_flag_and_paths() {
        let flag = EnvFlag::new("test.custom.key");
        assert_eq!(flag.get_value(|| "default_val".to_string()), "default_val");
        assert_eq!(flag.get_value_as_int(42), 42);
        assert!(!flag.get_value_as_bool(false));

        let asset_loc = get_asset_location("geoip.dat");
        assert!(!asset_loc.as_os_str().is_empty());

        let cfg_path = get_configuration_path();
        assert!(!cfg_path.as_os_str().is_empty());
    }

    #[test]
    fn test_cmdarg_parser() {
        let raw = ["xray", "run", "-c", "config.json", "--log=debug"];
        let arg = Arg::new(&raw);

        assert_eq!(arg.get(1), Some("run"));
        assert_eq!(arg.find_flag("-c"), Some("config.json"));
        assert_eq!(arg.find_flag("--log"), Some("debug"));
        assert_eq!(arg.len(), 5);
    }

    #[tokio::test]
    async fn test_behavior_seed_limited_drainer() {
        let mut drainer = BehaviorSeedLimitedDrainer::new(12345, 16, 64, 32);
        assert!(drainer.drain_size >= 16);

        let initial_drain = drainer.drain_size;
        drainer.acknowledge_receive(10);
        assert_eq!(drainer.drain_size, initial_drain - 10);

        let payload = vec![0x42u8; 500];
        let mut cursor = Cursor::new(payload);
        drainer.drain(&mut cursor).await.unwrap();
    }
}
