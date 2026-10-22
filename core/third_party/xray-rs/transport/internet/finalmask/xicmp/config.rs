// Module: transport\internet\finalmask\xicmp\config.rs
// 1:1 Rust implementation corresponding to Go transport\internet\finalmask\xicmp\config.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XIcmpConfig {
    #[serde(default)]
    pub ip: String,
    #[serde(default = "default_icmp_id")]
    pub id: u16,
    #[serde(default = "default_icmp_seq")]
    pub sequence: u16,
    #[serde(default)]
    pub payload_size: usize,
}

fn default_icmp_id() -> u16 {
    0x1234
}

fn default_icmp_seq() -> u16 {
    1
}

impl Default for XIcmpConfig {
    fn default() -> Self {
        Self {
            ip: String::new(),
            id: default_icmp_id(),
            sequence: default_icmp_seq(),
            payload_size: 64,
        }
    }
}
