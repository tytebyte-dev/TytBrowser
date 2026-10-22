// Module: proxy\shadowsocks\config.pb.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerConfig {
    pub users: Vec<String>,
    pub network: Vec<i32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientConfig {
    pub server: Vec<String>,
}
