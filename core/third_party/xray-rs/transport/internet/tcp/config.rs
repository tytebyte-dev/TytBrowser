// Module: transport\internet\tcp\config.rs
// 1:1 Rust implementation corresponding to Go transport\internet\tcp\config.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TcpConfig {
    #[serde(default)]
    pub accept_proxy_protocol: bool,
    #[serde(default)]
    pub header_type: Option<String>,
}

impl TcpConfig {
    pub fn new() -> Self {
        Self::default()
    }
}
