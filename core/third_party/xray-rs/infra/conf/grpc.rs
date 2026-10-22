// Module: infra\conf\grpc.rs
// 1:1 Rust implementation corresponding to Go infra\conf\grpc.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrpcConfig {
    #[serde(default)]
    pub service_name: String,
    #[serde(default)]
    pub multi_mode: bool,
    #[serde(default)]
    pub idle_timeout: Option<u32>,
    #[serde(default)]
    pub health_check_timeout: Option<u32>,
    #[serde(default)]
    pub permit_without_stream: Option<bool>,
    #[serde(default)]
    pub initial_windows_size: Option<i32>,
}
