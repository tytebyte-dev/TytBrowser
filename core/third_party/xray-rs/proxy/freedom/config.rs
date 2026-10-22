// Module: proxy\freedom\config.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FreedomConfig {
    #[serde(default)]
    pub domain_strategy: String,
    #[serde(default)]
    pub timeout: u32,
}
