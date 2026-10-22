// Module: infra\conf\router.rs
// 1:1 Rust implementation corresponding to Go infra\conf\router.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouterRuleConfig {
    #[serde(default)]
    pub outbound_tag: Option<String>,
    #[serde(default)]
    pub balancer_tag: Option<String>,
    #[serde(default)]
    pub domain: Option<Vec<String>>,
    #[serde(default)]
    pub ip: Option<Vec<String>>,
    #[serde(default)]
    pub port: Option<String>,
    #[serde(default)]
    pub network: Option<String>,
    #[serde(default)]
    pub protocol: Option<Vec<String>>,
    #[serde(default)]
    pub inbound_tag: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouterRuleConfigWrapper(pub RouterRuleConfig);

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouterConfig {
    #[serde(default)]
    pub domain_strategy: Option<String>,
    #[serde(default)]
    pub rules: Vec<RouterRuleConfig>,
}
