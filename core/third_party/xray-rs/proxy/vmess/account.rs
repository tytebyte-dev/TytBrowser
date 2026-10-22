// Module: proxy\vmess\account.rs
// 1:1 Rust implementation corresponding to Go proxy\vmess\account.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub alter_id: u32,
    pub security: String,
}

impl Account {
    pub fn new(id: impl Into<String>, alter_id: u32, security: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            alter_id,
            security: security.into(),
        }
    }
}
