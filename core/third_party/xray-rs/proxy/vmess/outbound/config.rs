// Module: proxy\vmess\outbound\config.rs
// 1:1 Rust implementation corresponding to Go proxy\vmess\outbound\config.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutboundConfig {
    pub receiver_address: String,
    pub receiver_port: u16,
}

impl OutboundConfig {
    pub fn receiver(&self) -> String {
        format!("{}:{}", self.receiver_address, self.receiver_port)
    }
}
