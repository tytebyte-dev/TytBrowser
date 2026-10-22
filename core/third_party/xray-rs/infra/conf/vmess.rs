// Module: infra\conf\vmess.rs
// 1:1 Rust implementation corresponding to Go infra\conf\vmess.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VmessClient {
    pub id: String,
    #[serde(default, alias = "alterId", alias = "alter_id")]
    pub alter_id: Option<u32>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub level: Option<u32>,
    #[serde(default)]
    pub security: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VmessDefaultConfig {
    #[serde(default, alias = "alterId", alias = "alter_id")]
    pub alter_id: Option<u32>,
    #[serde(default)]
    pub level: Option<u32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VmessServerConfig {
    #[serde(default)]
    pub clients: Vec<VmessClient>,
    #[serde(default)]
    pub default: Option<VmessDefaultConfig>,
}

pub type VmessInboundConfig = VmessServerConfig;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VmessOutboundTarget {
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub users: Vec<VmessClient>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VmessOutboundConfig {
    #[serde(default)]
    pub vnext: Vec<VmessOutboundTarget>,
}
