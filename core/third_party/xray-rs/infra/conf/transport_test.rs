// Module: infra\conf\transport_test.rs
// 1:1 Rust unit test suite corresponding to Go infra\conf\transport_test.go

#[cfg(test)]
mod tests {
    use super::super::transport_internet::StreamConfig;

    #[test]
    fn test_stream_config_parsing() {
        let json = r#"{"network": "ws", "security": "tls"}"#;
        let cfg: StreamConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.network, Some("ws".into()));
        assert_eq!(cfg.security, Some("tls".into()));
    }
}
