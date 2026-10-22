// Module: infra\conf\reverse.rs
// 1:1 Rust implementation corresponding to Go infra\conf\reverse.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReverseBridgeConfig {
    pub tag: String,
    pub domain: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReversePortalConfig {
    pub tag: String,
    pub domain: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReverseConfig {
    #[serde(default)]
    pub bridges: Vec<ReverseBridgeConfig>,
    #[serde(default)]
    pub portals: Vec<ReversePortalConfig>,
}
