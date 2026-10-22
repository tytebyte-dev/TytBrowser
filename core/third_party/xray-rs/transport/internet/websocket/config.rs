// Module: transport\internet\websocket\config.rs
// 1:1 Rust implementation corresponding to Go transport\internet\websocket\config.go

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebSocketConfig {
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub accept_proxy_protocol: bool,
    #[serde(default)]
    pub max_early_data: i32,
    #[serde(default)]
    pub use_browser_forwarding: bool,
    #[serde(default)]
    pub early_data_header_name: String,
}

impl WebSocketConfig {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            ..Default::default()
        }
    }
}
