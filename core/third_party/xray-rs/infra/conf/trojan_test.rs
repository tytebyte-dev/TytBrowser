// Module: infra\conf\trojan_test.rs
// Unit tests for Trojan inbound and outbound configuration parsing

#[cfg(test)]
mod tests {
    use super::super::trojan::{TrojanOutboundConfig, TrojanServerConfig};

    #[test]
    fn test_trojan_server_config_parsing() {
        let json = r#"{
            "clients": [
                {
                    "password": "secret_trojan_password",
                    "email": "user@trojan.net",
                    "level": 0
                }
            ],
            "fallbacks": [
                {
                    "dest": 80
                }
            ]
        }"#;

        let cfg: TrojanServerConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.clients.len(), 1);
        assert_eq!(cfg.clients[0].password, "secret_trojan_password");
        assert_eq!(cfg.fallbacks.unwrap().len(), 1);
    }

    #[test]
    fn test_trojan_outbound_config_parsing() {
        let json = r#"{
            "servers": [
                {
                    "address": "trojan.server.com",
                    "port": 443,
                    "password": "client_password"
                }
            ]
        }"#;

        let cfg: TrojanOutboundConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.servers.len(), 1);
        assert_eq!(cfg.servers[0].address, Some("trojan.server.com".into()));
        assert_eq!(cfg.servers[0].port, Some(443));
        assert_eq!(cfg.servers[0].password, Some("client_password".into()));
    }
}
