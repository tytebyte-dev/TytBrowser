// Module: infra\conf\metrics_test.rs
// 1:1 Rust unit test suite corresponding to Go infra\conf\metrics_test.go

#[cfg(test)]
mod tests {
    use super::super::metrics::MetricsConfig;

    #[test]
    fn test_metrics_config() {
        let cfg = MetricsConfig::new("metrics-tag");
        assert_eq!(cfg.tag, "metrics-tag");
    }
}
