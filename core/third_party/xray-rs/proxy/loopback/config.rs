// Module: proxy\loopback\config.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoopbackConfig {
    #[serde(default)]
    pub inbound_tag: String,
}
