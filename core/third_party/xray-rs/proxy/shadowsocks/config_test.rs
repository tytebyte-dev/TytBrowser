// Module: proxy\shadowsocks\config_test.rs
// 1:1 Rust unit test suite corresponding to Go proxy\shadowsocks\config_test.go

#[cfg(test)]
mod tests {
    use super::super::config::ShadowsocksConfig;
    use super::super::validator::MethodValidator;

    #[test]
    fn test_shadowsocks_config_and_validation() {
        let cfg = ShadowsocksConfig {
            method: "aes-256-gcm".into(),
            password: "secret_password".into(),
            network: Some("tcp,udp".into()),
            email: None,
        };
        assert!(MethodValidator::is_supported(&cfg.method));
        assert!(!MethodValidator::is_supported("des-cbc"));
    }
}
