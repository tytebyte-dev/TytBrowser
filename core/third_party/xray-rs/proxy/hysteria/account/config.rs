// Module: proxy\hysteria\account\config.rs
// 1:1 Rust implementation corresponding to Go proxy\hysteria\account\config.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    pub auth: String,
    pub user_level: u32,
}

impl Account {
    pub fn new(auth: &str) -> Self {
        Self {
            auth: auth.to_string(),
            user_level: 0,
        }
    }
}
