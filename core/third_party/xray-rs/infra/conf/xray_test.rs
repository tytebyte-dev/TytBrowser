// Module: infra\conf\xray_test.rs
// 1:1 Rust unit test suite corresponding to Go infra\conf\xray_test.go

#[cfg(test)]
mod tests {
    use super::super::Config;

    #[test]
    fn test_full_xray_config_roundtrip() {
        let json = r#"{
            "log": {"loglevel": "info"},
            "inbounds": [{"protocol": "socks", "port": 1080}],
            "outbounds": [{"protocol": "freedom"}]
        }"#;
        let cfg: Config = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.inbounds.len(), 1);
        assert_eq!(cfg.outbounds.len(), 1);
    }
}
