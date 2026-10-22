// Module: transport\internet\grpc\config_test.rs
// 1:1 Rust unit test suite corresponding to Go transport\internet\grpc\config_test.go

#[cfg(test)]
mod tests {
    use super::super::config::GrpcConfig;

    #[test]
    fn test_grpc_config_creation() {
        let cfg = GrpcConfig::new("GunService");
        assert_eq!(cfg.service_name, "GunService");
        assert_eq!(cfg.idle_timeout, 10);
        assert_eq!(cfg.health_check_timeout, 20);
    }
}
