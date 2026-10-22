// Module: infra\conf\vless_test.rs
// 1:1 Rust unit test suite corresponding to Go infra\conf\vless_test.go

#[cfg(test)]
mod tests {
    use super::super::vless::{VlessOutboundConfig, VlessServerConfig};

    #[test]
    fn test_vless_server_config_parsing() {
        let json = r#"{
            "clients": [
                {
                    "id": "a0000000-0000-0000-0000-000000000001",
                    "flow": "xtls-rprx-vision",
                    "email": "user@xray.com",
                    "level": 1
                }
            ],
            "decryption": "none",
            "fallbacks": [
                {
                    "dest": 80,
                    "xver": 1
                },
                {
                    "path": "/websocket",
                    "dest": 1234
                }
            ]
        }"#;

        let cfg: VlessServerConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.clients.len(), 1);
        assert_eq!(cfg.clients[0].flow, Some("xtls-rprx-vision".into()));
        assert_eq!(cfg.clients[0].email, Some("user@xray.com".into()));
        assert_eq!(cfg.decryption, Some("none".into()));

        let fallbacks = cfg.fallbacks.expect("Fallbacks should be parsed");
        assert_eq!(fallbacks.len(), 2);
        assert_eq!(fallbacks[0].xver, Some(1));
        assert_eq!(fallbacks[1].path, Some("/websocket".into()));
    }

    #[test]
    fn test_vless_outbound_config_parsing() {
        let json = r#"{
            "vnext": [
                {
                    "address": "example.com",
                    "port": 443,
                    "users": [
                        {
                            "id": "b0000000-0000-0000-0000-000000000002",
                            "flow": "xtls-rprx-vision",
                            "encryption": "none"
                        }
                    ]
                }
            ]
        }"#;

        let cfg: VlessOutboundConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.vnext.len(), 1);
        assert_eq!(cfg.vnext[0].address, Some("example.com".into()));
        assert_eq!(cfg.vnext[0].port, Some(443));
        assert_eq!(cfg.vnext[0].users.len(), 1);
        assert_eq!(cfg.vnext[0].users[0].flow, Some("xtls-rprx-vision".into()));
    }
}
