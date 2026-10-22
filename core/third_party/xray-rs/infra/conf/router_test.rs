// Module: infra\conf\router_test.rs
// 1:1 Rust unit test suite corresponding to Go infra\conf\router_test.go

#[cfg(test)]
mod tests {
    use super::super::router::RouterConfig;

    #[test]
    fn test_router_config_parsing() {
        let json = r#"{"domain_strategy": "IPIfNonMatch", "rules": [{"outbound_tag": "direct", "domain": ["geosite:cn"]}]}"#;
        let cfg: RouterConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.domain_strategy, Some("IPIfNonMatch".into()));
        assert_eq!(cfg.rules.len(), 1);
    }
}
