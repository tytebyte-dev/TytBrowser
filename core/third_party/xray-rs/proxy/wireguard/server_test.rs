// Module: proxy\wireguard\server_test.rs
// 1:1 Rust unit test suite corresponding to Go proxy\wireguard\server_test.go

#[cfg(test)]
mod tests {
    use super::super::config::{WireGuardConfig, WireGuardPeer};
    use super::super::wireguard::DEFAULT_MTU;

    #[test]
    fn test_wireguard_config_construction() {
        let peer = WireGuardPeer {
            public_key: "abc".into(),
            endpoint: "1.2.3.4:51820".into(),
            keepalive: 25,
        };
        let cfg = WireGuardConfig {
            secret_key: "def".into(),
            address: vec!["10.0.0.2/32".into()],
            peers: vec![peer],
            mtu: Some(DEFAULT_MTU),
            reserved: None,
        };
        assert_eq!(cfg.peers.len(), 1);
        assert_eq!(cfg.mtu, Some(1420));
    }
}
