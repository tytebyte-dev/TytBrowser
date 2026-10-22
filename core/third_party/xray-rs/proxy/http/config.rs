// Module: proxy\http\config.rs
// 1:1 Rust implementation corresponding to Go proxy\http\config.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerConfig {
    pub timeout_secs: u32,
    pub allow_transparent: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientConfig {
    pub server_address: String,
    pub server_port: u16,
}
