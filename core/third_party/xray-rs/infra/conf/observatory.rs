// Module: infra\conf\observatory.rs
// 1:1 Rust implementation corresponding to Go infra\conf\observatory.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservatoryConfig {
    #[serde(default)]
    pub subject_selectors: Vec<String>,
    #[serde(default)]
    pub probe_url: Option<String>,
    #[serde(default)]
    pub probe_interval: Option<String>,
    #[serde(default)]
    pub enable_concurrency: Option<bool>,
}
