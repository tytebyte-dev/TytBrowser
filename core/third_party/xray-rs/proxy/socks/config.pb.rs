// Module: proxy\socks\config.pb.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerConfig {
    pub auth_type: i32,
    pub accounts: Vec<String>,
    pub address: Option<String>,
    pub udp_enabled: bool,
    pub timeout: u32,
    pub user_level: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientConfig {
    pub server: Vec<String>,
    pub version: i32,
}
