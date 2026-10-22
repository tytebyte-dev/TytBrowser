// Module: infra\conf\blackhole_test.rs
// 1:1 Rust unit test suite corresponding to Go infra\conf\blackhole_test.go

#[cfg(test)]
mod tests {
    use super::super::blackhole::BlackholeConfig;

    #[test]
    fn test_blackhole_json_parsing_none() {
        let json = r#"{"response": {"type": "none"}}"#;
        let cfg: BlackholeConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.response_type(), Some("none"));
    }

    #[test]
    fn test_blackhole_json_parsing_http() {
        let json = r#"{"response": {"type": "http"}}"#;
        let cfg: BlackholeConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.response_type(), Some("http"));
    }
}
