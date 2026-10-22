// Module: infra\conf\socks.rs
// 1:1 Rust implementation corresponding to Go infra\conf\socks.go

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    #[serde(default)]
    pub user_level: Option<u32>,
}
