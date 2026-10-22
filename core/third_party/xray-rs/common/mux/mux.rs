// Module: common\mux\mux.rs
// 1:1 Rust implementation corresponding to Go common\mux\mux.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientStrategy {
    pub max_concurrency: u32,
    pub max_connection: u32,
}

impl Default for ClientStrategy {
    fn default() -> Self {
        Self {
            max_concurrency: 8,
            max_connection: 128,
        }
    }
}
