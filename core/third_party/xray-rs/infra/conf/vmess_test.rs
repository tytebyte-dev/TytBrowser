// Module: infra\conf\vmess_test.rs
// 1:1 Rust unit test suite corresponding to Go infra\conf\vmess_test.go

#[cfg(test)]
mod tests {
    use super::super::vmess::{VmessOutboundConfig, VmessServerConfig};

    #[test]
    fn test_vmess_server_config_parsing() {
        let json = r#"{
            "clients": [
                {
                    "id": "b0000000-0000-0000-0000-000000000002",
                    "alterId": 0,
                    "email": "user@vmess.org",
                    "security": "auto"
                }
            ],
            "default": {
                "alterId": 0,
                "level": 0
            }
        }"#;

        let cfg: VmessServerConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.clients.len(), 1);
        assert_eq!(cfg.clients[0].alter_id, Some(0));
        assert_eq!(cfg.clients[0].email, Some("user@vmess.org".into()));
        assert_eq!(cfg.default.unwrap().alter_id, Some(0));
    }

    #[test]
    fn test_vmess_outbound_config_parsing() {
        let json = r#"{
            "vnext": [
                {
                    "address": "vmess.example.com",
                    "port": 443,
                    "users": [
                        {
                            "id": "c0000000-0000-0000-0000-000000000003",
                            "security": "aes-128-gcm"
                        }
                    ]
                }
            ]
        }"#;

        let cfg: VmessOutboundConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.vnext.len(), 1);
        assert_eq!(cfg.vnext[0].address, Some("vmess.example.com".into()));
        assert_eq!(cfg.vnext[0].users[0].security, Some("aes-128-gcm".into()));
    }
}
