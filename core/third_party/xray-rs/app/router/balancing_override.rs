// Module: app\router\balancing_override.rs
// 1:1 Rust implementation corresponding to Go app\router\balancing_override.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BalancingOverride {
    pub tag: String,
    pub target: String,
}

impl BalancingOverride {
    pub fn new(tag: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            target: target.into(),
        }
    }
}
