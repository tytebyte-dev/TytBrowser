// Module: infra\conf\transport_internet.rs
// 1:1 Rust implementation corresponding to Go infra\conf\transport_internet.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamConfig {
    #[serde(default)]
    pub network: Option<String>,
    #[serde(default)]
    pub security: Option<String>,
    #[serde(default)]
    pub tls_settings: Option<serde_json::Value>,
    #[serde(default)]
    pub tcp_settings: Option<serde_json::Value>,
    #[serde(default)]
    pub ws_settings: Option<serde_json::Value>,
}
