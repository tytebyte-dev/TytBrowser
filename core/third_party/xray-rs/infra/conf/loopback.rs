// Module: infra\conf\loopback.rs
// 1:1 Rust implementation corresponding to Go infra\conf\loopback.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoopbackConfig {
    #[serde(default)]
    pub inbound_tag: String,
}

impl LoopbackConfig {
    pub fn new(inbound_tag: impl Into<String>) -> Self {
        Self {
            inbound_tag: inbound_tag.into(),
        }
    }
}
