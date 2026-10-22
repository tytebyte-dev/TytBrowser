// Module: proxy\wireguard\config.rs
// 1:1 Rust implementation corresponding to Go proxy\wireguard\config.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireGuardPeer {
    pub public_key: String,
    pub endpoint: String,
    #[serde(default)]
    pub keepalive: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireGuardConfig {
    pub secret_key: String,
    pub address: Vec<String>,
    pub peers: Vec<WireGuardPeer>,
    #[serde(default)]
    pub mtu: Option<u32>,
    #[serde(default)]
    pub reserved: Option<Vec<u8>>,
}
