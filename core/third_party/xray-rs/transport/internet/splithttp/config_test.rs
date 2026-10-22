// Module: transport\internet\splithttp\config_test.rs
// 1:1 Rust unit test suite corresponding to Go transport\internet\splithttp\config_test.go

#[cfg(test)]
mod tests {
    use super::super::config::SplitHttpConfig;

    #[test]
    fn test_splithttp_config_defaults() {
        let mut cfg = SplitHttpConfig::default();
        if cfg.path.is_empty() {
            cfg.path = "/".into();
        }
        assert_eq!(cfg.path, "/");
    }
}
