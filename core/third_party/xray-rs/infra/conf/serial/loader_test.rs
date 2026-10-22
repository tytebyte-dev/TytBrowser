// Module: infra\conf\serial\loader_test.rs
// 1:1 Rust unit test suite corresponding to Go infra\conf\serial\loader_test.go

#[cfg(test)]
mod tests {
    use super::super::builder::{build_config_from_slices, build_json_config, merge_configs};
    use crate::infra::conf::Config;

    #[test]
    fn test_build_json_config() {
        let json = r#"{"log": {"loglevel": "warning"}}"#;
        let val = build_json_config(json).expect("valid json");
        assert_eq!(val["log"]["loglevel"], "warning");
    }

    #[test]
    fn test_merge_configs() {
        let json1 = r#"{"log": {"loglevel": "warning"}, "inbounds": [{"protocol": "socks"}]}"#;
        let json2 = r#"{"outbounds": [{"protocol": "freedom"}]}"#;
        let merged = build_config_from_slices(&[json1, json2]).expect("valid merged config");
        assert_eq!(merged.inbounds.len(), 1);
        assert_eq!(merged.outbounds.len(), 1);
        assert_eq!(merged.log.unwrap().loglevel.as_deref(), Some("warning"));

        let empty = merge_configs(&[Config::default()]);
        assert!(empty.inbounds.is_empty());
    }
}
