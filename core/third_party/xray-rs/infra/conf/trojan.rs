// Module: infra\conf\trojan.rs
// 1:1 Rust implementation corresponding to Go infra\conf\trojan.go

use super::vless::VlessFallback;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrojanClient {
    pub password: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub level: Option<u32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrojanServerConfig {
    #[serde(default)]
    pub clients: Vec<TrojanClient>,
    #[serde(default)]
    pub fallbacks: Option<Vec<VlessFallback>>,
}

pub type TrojanInboundConfig = TrojanServerConfig;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrojanServerTarget {
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub level: Option<u32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrojanOutboundConfig {
    #[serde(default)]
    pub servers: Vec<TrojanServerTarget>,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub password: Option<String>,
}
