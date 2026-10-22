// Module: infra\conf\dokodemo_test.rs
// 1:1 Rust unit test suite corresponding to Go infra\conf\dokodemo_test.go

#[cfg(test)]
mod tests {
    use super::super::dokodemo::DokodemoDoorConfig;

    #[test]
    fn test_dokodemo_json_parsing() {
        let json = r#"{
            "address": "1.1.1.1",
            "port": 53,
            "network": "tcp,udp",
            "timeout": 30,
            "followRedirect": true,
            "userLevel": 1
        }"#;
        let cfg: DokodemoDoorConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.address, Some("1.1.1.1".into()));
        assert_eq!(cfg.port, Some(53));
        assert_eq!(cfg.network, Some("tcp,udp".into()));
        assert_eq!(cfg.timeout, Some(30));
        assert_eq!(cfg.follow_redirect, Some(true));
        assert_eq!(cfg.user_level, Some(1));
    }
}
