// Module: proxy\blackhole\config_test.rs
// 1:1 Rust unit test suite corresponding to Go proxy\blackhole\config_test.go

#[cfg(test)]
mod tests {
    use super::super::config::{BlackholeConfig, ResponseType};

    #[test]
    fn test_blackhole_config_defaults() {
        let cfg = BlackholeConfig::default();
        assert_eq!(cfg.response, ResponseType::None);
    }
}
