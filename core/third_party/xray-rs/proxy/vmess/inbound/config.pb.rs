// Module: proxy\vmess\inbound\config.pb.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub user: Vec<String>,
    pub default: Option<String>,
    pub secure_encryption_only: bool,
}
