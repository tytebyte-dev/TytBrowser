// Module: proxy\vmess\inbound\config.rs
// 1:1 Rust implementation corresponding to Go proxy\vmess\inbound\config.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InboundConfig {
    #[serde(default)]
    pub secure_encryption_only: bool,
    #[serde(default)]
    pub users: Vec<String>,
}

impl InboundConfig {
    pub fn new() -> Self {
        Self::default()
    }
}
