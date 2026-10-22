// Module: app\dns\fakedns\fakedns.pb.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FakeDnsPool {
    pub ip_pool: String,
    pub lru_size: i64,
}
