// Module: infra\conf\socks_test.rs
// 1:1 Rust unit test suite corresponding to Go infra\conf\socks_test.go

#[cfg(test)]
mod tests {
    use super::super::socks::SocksServerConfig;

    #[test]
    fn test_socks_config_parsing() {
        let json = r#"{"auth": "noauth", "udp": true}"#;
        let cfg: SocksServerConfig = serde_json::from_str(json).unwrap();
        assert!(cfg.udp);
    }
}
