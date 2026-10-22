// Module: infra\conf\general_test.rs
// 1:1 Rust unit test suite corresponding to Go infra\conf\general_test.go

#[cfg(test)]
mod tests {
    use super::super::Config;

    #[test]
    fn test_empty_config_parse() {
        let json = "{}";
        let cfg: Config = serde_json::from_str(json).unwrap();
        assert!(cfg.inbounds.is_empty());
        assert!(cfg.outbounds.is_empty());
    }
}
