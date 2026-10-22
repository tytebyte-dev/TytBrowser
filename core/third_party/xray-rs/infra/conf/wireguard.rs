// Module: infra\conf\wireguard.rs
// 1:1 Rust implementation corresponding to Go infra\conf\wireguard.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireGuardPeerConfig {
    #[serde(alias = "publicKey", alias = "public_key")]
    pub public_key: String,
    #[serde(default, alias = "preSharedKey", alias = "preshared_key")]
    pub preshared_key: Option<String>,
    #[serde(default)]
    pub endpoint: String,
    #[serde(default, alias = "keepAlive", alias = "keepalive")]
    pub keepalive: Option<u16>,
    #[serde(default, alias = "allowedIPs", alias = "allowed_ips")]
    pub allowed_ips: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireGuardConfig {
    #[serde(alias = "secretKey", alias = "secret_key")]
    pub secret_key: String,
    #[serde(default)]
    pub address: Vec<String>,
    #[serde(default)]
    pub peers: Vec<WireGuardPeerConfig>,
    #[serde(default, alias = "mtu", alias = "MTU")]
    pub mtu: Option<u32>,
    #[serde(default, alias = "workers", alias = "num_workers")]
    pub workers: Option<u32>,
    #[serde(default)]
    pub reserved: Option<Vec<u8>>,
    #[serde(default, alias = "noKernelTun")]
    pub no_kernel_tun: Option<bool>,
}
