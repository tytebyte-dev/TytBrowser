// Module: app\stats\config.pb.rs
// 1:1 Rust protobuf message definitions corresponding to Go app\stats\config.pb.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub enable_stats: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelConfig {
    #[serde(rename = "Blocking", default)]
    pub blocking: bool,
    #[serde(rename = "SubscriberLimit", default)]
    pub subscriber_limit: i32,
    #[serde(rename = "BufferSize", default)]
    pub buffer_size: i32,
}

impl ChannelConfig {
    pub fn new(blocking: bool, subscriber_limit: i32, buffer_size: i32) -> Self {
        Self {
            blocking,
            subscriber_limit,
            buffer_size,
        }
    }
}
