// Module: app\version\config.pb.rs
// 1:1 Rust protobuf message definitions corresponding to Go app\version\config.pb.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "core_version", alias = "coreVersion", default)]
    pub core_version: String,
    #[serde(rename = "min_version", alias = "minVersion", default)]
    pub min_version: String,
    #[serde(rename = "max_version", alias = "maxVersion", default)]
    pub max_version: String,
}
