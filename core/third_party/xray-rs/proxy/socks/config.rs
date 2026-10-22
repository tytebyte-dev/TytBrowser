// Module: proxy\socks\config.rs
// 1:1 Rust implementation corresponding to Go proxy\socks\config.go

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SocksServerConfig {
    #[serde(default)]
    pub auth: Option<String>,
    #[serde(default)]
    pub accounts: HashMap<String, String>,
    #[serde(default)]
    pub udp: bool,
    #[serde(default)]
    pub ip: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SocksClientConfig {
    pub address: String,
    pub port: u16,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub pass: Option<String>,
}
