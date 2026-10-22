// Module: proxy\trojan\config.rs
// 1:1 Rust implementation corresponding to Go proxy\trojan\config.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrojanAccount {
    pub password: String,
    #[serde(default)]
    pub email: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrojanServerConfig {
    pub clients: Vec<TrojanAccount>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrojanClientConfig {
    pub address: String,
    pub port: u16,
    pub password: String,
}
