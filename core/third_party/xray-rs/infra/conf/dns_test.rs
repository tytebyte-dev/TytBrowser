// Module: infra\conf\dns_test.rs
// 1:1 Rust unit test suite corresponding to Go infra\conf\dns_test.go

#[cfg(test)]
mod tests {
    use super::super::dns::DnsConfig;

    #[test]
    fn test_dns_config_parsing() {
        let json = r#"{"client_ip": "1.1.1.1", "query_strategy": "UseIP"}"#;
        let cfg: DnsConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.client_ip, Some("1.1.1.1".into()));
    }
}
