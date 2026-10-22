// Module: infra\conf\log.rs
// 1:1 Rust implementation corresponding to Go infra\conf\log.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogConfig {
    #[serde(default)]
    pub access: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub loglevel: Option<String>,
    #[serde(default)]
    pub mask_address: Option<String>,
}
