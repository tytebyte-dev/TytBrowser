// Module: transport\internet\httpupgrade\config.rs
// 1:1 Rust implementation corresponding to Go transport\internet\httpupgrade\config.go

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpUpgradeConfig {
    #[serde(default)]
    pub host: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub accept_proxy_protocol: bool,
}

impl HttpUpgradeConfig {
    pub fn new(host: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            path: path.into(),
            ..Default::default()
        }
    }

    pub fn get_normalized_path(&self) -> String {
        if self.path.is_empty() {
            return "/".to_string();
        }
        if !self.path.starts_with('/') {
            return format!("/{}", self.path);
        }
        self.path.clone()
    }
}
