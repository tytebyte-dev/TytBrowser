// Module: app\observatory\burst\config.pb.rs
// 1:1 Rust protobuf message definitions corresponding to Go app\observatory\burst\config.pb.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthPingConfig {
    #[serde(default)]
    pub destination: String,
    #[serde(default)]
    pub connectivity: String,
    #[serde(default)]
    pub interval: i64,
    #[serde(default)]
    pub sampling_count: i32,
    #[serde(default)]
    pub timeout: i64,
    #[serde(default)]
    pub http_method: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub subject_selector: Vec<String>,
    #[serde(default)]
    pub ping_config: Option<HealthPingConfig>,
}
