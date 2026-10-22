// Module: app\dns\fakedns\fakedns.rs
// 1:1 Rust implementation corresponding to Go app\dns\fakedns\fakedns.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FakeDnsConfig {
    pub ip_pool: String,
    pub pool_size: usize,
}

impl Default for FakeDnsConfig {
    fn default() -> Self {
        Self {
            ip_pool: "198.18.0.0/15".into(),
            pool_size: 65535,
        }
    }
}
