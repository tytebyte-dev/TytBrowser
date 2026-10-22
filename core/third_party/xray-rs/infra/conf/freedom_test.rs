// Module: infra\conf\freedom_test.rs
// 1:1 Rust unit test suite corresponding to Go infra\conf\freedom_test.go

#[cfg(test)]
mod tests {
    use super::super::freedom::FreedomConfig;

    #[test]
    fn test_freedom_json_parsing_comprehensive() {
        let json = r#"{
            "domainStrategy": "UseIP4",
            "userLevel": 1,
            "fragment": {
                "packets": "1-3",
                "length": "100-200",
                "interval": "10-20"
            },
            "noise": {
                "type": "rand",
                "packet": "50-100",
                "delay": "10-20"
            },
            "proxyProtocol": 2
        }"#;

        let cfg: FreedomConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.domain_strategy, Some("UseIP4".into()));
        assert_eq!(cfg.user_level, Some(1));
        assert_eq!(cfg.proxy_protocol, Some(2));

        let frag = cfg.fragment.expect("Fragment should be present");
        assert_eq!(frag.packets, Some("1-3".into()));
        assert_eq!(frag.length, Some("100-200".into()));

        let noise = cfg.noise.expect("Noise should be present");
        assert_eq!(noise.noise_type, Some("rand".into()));
    }
}
