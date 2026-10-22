// Module: proxy\hysteria\config.pb.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub version: u32,
    pub auth: String,
    pub up_mbps: u64,
    pub down_mbps: u64,
}
