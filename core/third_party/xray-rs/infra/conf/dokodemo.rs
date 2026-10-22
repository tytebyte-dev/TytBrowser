// Module: infra\conf\dokodemo.rs
// 1:1 Rust implementation corresponding to Go infra\conf\dokodemo.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DokodemoDoorConfig {
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub network: Option<String>,
    #[serde(default)]
    pub timeout: Option<u32>,
    #[serde(default, alias = "followRedirect", alias = "follow_redirect")]
    pub follow_redirect: Option<bool>,
    #[serde(default, alias = "userLevel", alias = "user_level")]
    pub user_level: Option<u32>,
}
