// Module: proxy\http\config.pb.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerConfig {
    pub timeout: u32,
    pub accounts: Vec<String>,
    pub allow_transparent: bool,
    pub user_level: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientConfig {
    pub server: Vec<String>,
    pub user_level: u32,
}
