// Module: proxy\hysteria\config.rs
// 1:1 Rust implementation corresponding to Go proxy\hysteria\config.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HysteriaConfig {
    #[serde(default)]
    pub auth: String,
    #[serde(default = "default_speed")]
    pub up_mbps: u64,
    #[serde(default = "default_speed")]
    pub down_mbps: u64,
}

fn default_speed() -> u64 {
    100
}

impl HysteriaConfig {
    pub fn new(auth: impl Into<String>, up_mbps: u64, down_mbps: u64) -> Self {
        Self {
            auth: auth.into(),
            up_mbps,
            down_mbps,
        }
    }
}
