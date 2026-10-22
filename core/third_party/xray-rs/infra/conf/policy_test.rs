// Module: infra\conf\policy_test.rs
// 1:1 Rust unit test suite corresponding to Go infra\conf\policy_test.go

#[cfg(test)]
mod tests {
    use super::super::policy::PolicyConfig;

    #[test]
    fn test_policy_config_parsing() {
        let json = r#"{"levels": {"0": {"conn_idle": 300}}}"#;
        let cfg: PolicyConfig = serde_json::from_str(json).unwrap();
        assert!(cfg.levels.contains_key("0"));
    }
}
