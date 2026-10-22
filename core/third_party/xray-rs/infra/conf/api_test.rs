// Module: infra\conf\api_test.rs
// 1:1 Rust unit test suite corresponding to Go infra\conf\api_test.go

#[cfg(test)]
mod tests {
    use super::super::api::ApiConfig;

    #[test]
    fn test_api_config_builder() {
        let cfg = ApiConfig::new("api-inbound")
            .with_service("HandlerService")
            .with_service("StatsService");
        assert_eq!(cfg.tag, "api-inbound");
        assert_eq!(cfg.services.len(), 2);
    }
}
