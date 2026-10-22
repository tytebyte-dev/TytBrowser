// Module: app\metrics\config.pb.rs
// 1:1 Rust protobuf message definitions corresponding to Go app\metrics\config.pb.go

use serde::{Deserialize, Serialize};

/// Config is the settings for metrics.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    /// Tag of the outbound handler that handles metrics http connections.
    #[serde(default)]
    pub tag: String,
    /// Listen address for metrics HTTP server (e.g. "127.0.0.1:8080").
    #[serde(default)]
    pub listen: String,
}

impl Config {
    pub fn new(tag: impl Into<String>, listen: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            listen: listen.into(),
        }
    }
}
