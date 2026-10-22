// Module: transport\internet\hysteria\config.rs
// 1:1 Rust implementation corresponding to Go transport\internet\hysteria\config.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HysteriaTransportConfig {
    #[serde(default)]
    pub auth: String,
    #[serde(default = "default_window")]
    pub recv_window_conn: u64,
    #[serde(default = "default_window")]
    pub recv_window_client: u64,
    #[serde(default = "default_max_conn")]
    pub max_conn_client: usize,
}

fn default_window() -> u64 {
    16 * 1024 * 1024
}

fn default_max_conn() -> usize {
    1024
}

impl Default for HysteriaTransportConfig {
    fn default() -> Self {
        Self {
            auth: String::new(),
            recv_window_conn: default_window(),
            recv_window_client: default_window(),
            max_conn_client: default_max_conn(),
        }
    }
}
