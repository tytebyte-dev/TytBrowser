// Module: app\observatory\config.pb.rs
// 1:1 Rust protobuf message definitions corresponding to Go app\observatory\config.pb.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthPingMeasurementResult {
    #[serde(default)]
    pub all: i64,
    #[serde(default)]
    pub fail: i64,
    #[serde(default)]
    pub deviation: i64,
    #[serde(default)]
    pub average: i64,
    #[serde(default)]
    pub max: i64,
    #[serde(default)]
    pub min: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutboundStatus {
    #[serde(default)]
    pub alive: bool,
    #[serde(default)]
    pub delay: i64,
    #[serde(default)]
    pub last_error_reason: String,
    #[serde(default)]
    pub outbound_tag: String,
    #[serde(default)]
    pub last_seen_time: i64,
    #[serde(default)]
    pub last_try_time: i64,
    #[serde(default)]
    pub health_ping: Option<HealthPingMeasurementResult>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservationResult {
    #[serde(default)]
    pub status: Vec<OutboundStatus>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProbeResult {
    #[serde(default)]
    pub alive: bool,
    #[serde(default)]
    pub delay: i64,
    #[serde(default)]
    pub last_error_reason: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Intensity {
    #[serde(default)]
    pub probe_interval: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub subject_selector: Vec<String>,
    #[serde(default)]
    pub probe_url: String,
    #[serde(default)]
    pub probe_interval: i64,
    #[serde(default)]
    pub enable_concurrency: bool,
}
