// Module: infra\conf\wireguard_test.rs
// 1:1 Rust unit test suite corresponding to Go infra\conf\wireguard_test.go

#[cfg(test)]
mod tests {
    use super::super::wireguard::WireGuardConfig;

    #[test]
    fn test_wireguard_config_parsing_snake_case() {
        let json = r#"{
            "secret_key": "sec123",
            "address": ["10.0.0.2/24", "fd00::2/64"],
            "peers": [
                {
                    "public_key": "pub123",
                    "endpoint": "198.51.100.1:51820",
                    "keepalive": 25,
                    "allowed_ips": ["0.0.0.0/0", "::0/0"]
                }
            ],
            "mtu": 1420
        }"#;

        let cfg: WireGuardConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.secret_key, "sec123");
        assert_eq!(cfg.address.len(), 2);
        assert_eq!(cfg.peers.len(), 1);
        assert_eq!(cfg.peers[0].public_key, "pub123");
        assert_eq!(cfg.peers[0].keepalive, Some(25));
        assert_eq!(cfg.mtu, Some(1420));
    }

    #[test]
    fn test_wireguard_config_parsing_camel_case() {
        let json = r#"{
            "secretKey": "secCamel",
            "address": ["10.0.0.3/32"],
            "peers": [
                {
                    "publicKey": "pubCamel",
                    "endpoint": "203.0.113.1:51820",
                    "keepAlive": 21,
                    "allowedIPs": ["10.0.0.0/8"]
                }
            ],
            "noKernelTun": true,
            "workers": 4
        }"#;

        let cfg: WireGuardConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.secret_key, "secCamel");
        assert_eq!(cfg.peers[0].public_key, "pubCamel");
        assert_eq!(cfg.peers[0].keepalive, Some(21));
        assert_eq!(cfg.no_kernel_tun, Some(true));
        assert_eq!(cfg.workers, Some(4));
    }
}
