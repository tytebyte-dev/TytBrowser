// Module: infra\conf\http_test.rs
// 1:1 Rust unit test suite corresponding to Go infra\conf\http_test.go

#[cfg(test)]
mod tests {
    use super::super::http::HttpServerConfig;

    #[test]
    fn test_http_config_parsing() {
        let json = r#"{"allow_transparent": true}"#;
        let cfg: HttpServerConfig = serde_json::from_str(json).unwrap();
        assert!(cfg.allow_transparent);
    }
}
