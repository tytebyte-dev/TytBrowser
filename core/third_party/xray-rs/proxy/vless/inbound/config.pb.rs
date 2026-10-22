// Module: proxy\vless\inbound\config.pb.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fallback {
    pub name: String,
    pub alpn: String,
    pub path: String,
    pub r#type: String,
    pub dest: String,
    pub xver: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub clients: Vec<String>,
    pub decryption: String,
    pub fallbacks: Vec<Fallback>,
}
