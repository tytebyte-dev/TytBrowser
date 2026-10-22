// Module: infra\conf\shadowsocks.rs
// 1:1 Rust implementation corresponding to Go infra\conf\shadowsocks.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowsocksServerConfig {
    #[serde(default)]
    pub method: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub level: Option<u32>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub network: Option<String>,
    #[serde(default)]
    pub iv_check: Option<bool>,
}

pub type ShadowsocksInboundConfig = ShadowsocksServerConfig;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowsocksServerTarget {
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub level: Option<u32>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub uot: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowsocksOutboundConfig {
    #[serde(default)]
    pub servers: Vec<ShadowsocksServerTarget>,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub uot: Option<bool>,
}
