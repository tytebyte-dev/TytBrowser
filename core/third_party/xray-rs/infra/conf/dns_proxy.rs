// Module: infra\conf\dns_proxy.rs
// 1:1 Rust implementation corresponding to Go infra\conf\dns_proxy.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DnsProxyConfig {
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub network: Option<String>,
    #[serde(default)]
    pub user_level: Option<u32>,
}
