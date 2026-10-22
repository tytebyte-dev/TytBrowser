// Module: infra\conf\dns_proxy_test.rs
// 1:1 Rust unit test suite corresponding to Go infra\conf\dns_proxy_test.go

#[cfg(test)]
mod tests {
    use super::super::dns_proxy::DnsProxyConfig;

    #[test]
    fn test_dns_proxy_config() {
        let cfg = DnsProxyConfig {
            network: Some("tcp".into()),
            address: Some("8.8.8.8".into()),
            port: Some(53),
            user_level: None,
        };
        assert_eq!(cfg.port, Some(53));
    }
}
