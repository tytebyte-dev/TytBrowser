// Module: infra\conf\dns.rs
// 1:1 Rust implementation corresponding to Go infra\conf\dns.go

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DnsConfig {
    #[serde(default)]
    pub servers: Vec<serde_json::Value>,
    #[serde(default)]
    pub hosts: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub client_ip: Option<String>,
    #[serde(default)]
    pub query_strategy: Option<String>,
}
