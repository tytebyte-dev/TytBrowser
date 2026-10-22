// Module: infra\conf\fakedns.rs
// 1:1 Rust implementation corresponding to Go infra\conf\fakedns.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FakeDnsConfig {
    #[serde(default = "default_fakedns_ip_pool")]
    pub ip_pool: String,
    #[serde(default = "default_fakedns_pool_size")]
    pub pool_size: usize,
}

fn default_fakedns_ip_pool() -> String {
    "198.18.0.0/15".into()
}

fn default_fakedns_pool_size() -> usize {
    65535
}
