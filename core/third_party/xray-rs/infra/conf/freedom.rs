// Module: infra\conf\freedom.rs
// 1:1 Rust implementation corresponding to Go infra\conf\freedom.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Int32Range {
    #[serde(default)]
    pub from: i32,
    #[serde(default)]
    pub to: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FragmentConfig {
    #[serde(default)]
    pub packets: Option<String>,
    #[serde(default)]
    pub length: Option<String>,
    #[serde(default)]
    pub interval: Option<String>,
    #[serde(default, alias = "maxSplit", alias = "max_split")]
    pub max_split: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoiseConfig {
    #[serde(default, rename = "type")]
    pub noise_type: Option<String>,
    #[serde(default)]
    pub packet: Option<String>,
    #[serde(default)]
    pub delay: Option<String>,
    #[serde(default, alias = "applyTo", alias = "apply_to")]
    pub apply_to: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FreedomConfig {
    #[serde(default, alias = "domainStrategy", alias = "domain_strategy")]
    pub domain_strategy: Option<String>,
    #[serde(default, alias = "targetStrategy", alias = "target_strategy")]
    pub target_strategy: Option<String>,
    #[serde(default)]
    pub timeout: Option<u32>,
    #[serde(default)]
    pub redirect: Option<String>,
    #[serde(default, alias = "userLevel", alias = "user_level")]
    pub user_level: Option<u32>,
    #[serde(default)]
    pub fragment: Option<FragmentConfig>,
    #[serde(default)]
    pub noise: Option<NoiseConfig>,
    #[serde(default)]
    pub noises: Option<Vec<NoiseConfig>>,
    #[serde(default, alias = "proxyProtocol", alias = "proxy_protocol")]
    pub proxy_protocol: Option<u32>,
}
