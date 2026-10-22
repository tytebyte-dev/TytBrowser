// Module: proxy\shadowsocks_2022\config.pb.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerConfig {
    pub method: String,
    pub key: String,
    pub email: String,
    pub level: i32,
    pub network: Vec<i32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientConfig {
    pub address: Option<String>,
    pub port: u32,
    pub method: String,
    pub key: String,
}
