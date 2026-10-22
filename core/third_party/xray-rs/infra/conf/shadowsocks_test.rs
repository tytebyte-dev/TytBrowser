// Module: infra\conf\shadowsocks_test.rs
// 1:1 Rust unit test suite corresponding to Go infra\conf\shadowsocks_test.go

#[cfg(test)]
mod tests {
    use super::super::shadowsocks::{ShadowsocksOutboundConfig, ShadowsocksServerConfig};

    #[test]
    fn test_shadowsocks_server_config_parsing() {
        let json = r#"{
            "method": "aes-256-gcm",
            "password": "secret_password",
            "level": 0,
            "email": "ss@xray.com",
            "network": "tcp,udp",
            "iv_check": true
        }"#;

        let cfg: ShadowsocksServerConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.method, "aes-256-gcm");
        assert_eq!(cfg.password, "secret_password");
        assert_eq!(cfg.network, Some("tcp,udp".into()));
        assert_eq!(cfg.iv_check, Some(true));
    }

    #[test]
    fn test_shadowsocks_outbound_config_parsing() {
        let json = r#"{
            "servers": [
                {
                    "address": "127.0.0.1",
                    "port": 8388,
                    "method": "chacha20-ietf-poly1305",
                    "password": "outbound_password",
                    "uot": true
                }
            ]
        }"#;

        let cfg: ShadowsocksOutboundConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.servers.len(), 1);
        assert_eq!(cfg.servers[0].method, Some("chacha20-ietf-poly1305".into()));
        assert_eq!(cfg.servers[0].uot, Some(true));
    }
}
