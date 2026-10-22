// Module: proxy\wireguard\config.pb.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerConfig {
    pub public_key: String,
    pub pre_shared_key: String,
    pub endpoint: String,
    pub keep_alive: u32,
    pub allowed_ips: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceConfig {
    pub secret_key: String,
    pub endpoint: Vec<String>,
    pub peers: Vec<PeerConfig>,
    pub mtu: i32,
    pub num_workers: i32,
    pub reserved: Vec<u8>,
}
