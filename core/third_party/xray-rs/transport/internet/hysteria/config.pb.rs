// Module: transport\internet\hysteria\config.pb.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub version: u32,
    pub auth: String,
    pub recv_window_conn: u64,
    pub recv_window: u64,
    pub max_idle_timeout: u32,
}
