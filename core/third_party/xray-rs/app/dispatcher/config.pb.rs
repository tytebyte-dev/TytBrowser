// Module: app\dispatcher\config.pb.rs
// 1:1 Rust protobuf message definitions corresponding to Go app\dispatcher\config.proto

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionConfig {}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub settings: Option<SessionConfig>,
}
