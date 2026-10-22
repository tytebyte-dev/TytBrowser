// Module: infra\conf\vless.rs
// 1:1 Rust implementation corresponding to Go infra\conf\vless.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VlessClient {
    pub id: String,
    #[serde(default)]
    pub flow: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub level: Option<u32>,
    #[serde(default)]
    pub encryption: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VlessFallback {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub alpn: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub dest: Option<serde_json::Value>,
    #[serde(default)]
    pub xver: Option<u64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VlessServerConfig {
    #[serde(default)]
    pub clients: Vec<VlessClient>,
    #[serde(default)]
    pub decryption: Option<String>,
    #[serde(default)]
    pub fallbacks: Option<Vec<VlessFallback>>,
}

pub type VlessInboundConfig = VlessServerConfig;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VlessOutboundTarget {
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub users: Vec<VlessClient>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VlessOutboundConfig {
    #[serde(default)]
    pub vnext: Vec<VlessOutboundTarget>,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub flow: Option<String>,
    #[serde(default)]
    pub encryption: Option<String>,
}
