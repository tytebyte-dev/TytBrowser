// Module: infra\conf\http.rs
// 1:1 Rust implementation corresponding to Go infra\conf\http.go

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpServerConfig {
    #[serde(default)]
    pub accounts: HashMap<String, String>,
    #[serde(default)]
    pub allow_transparent: bool,
    #[serde(default)]
    pub user_level: Option<u32>,
}
