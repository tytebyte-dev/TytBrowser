// Module: transport\internet\grpc\config.rs
// 1:1 Rust implementation corresponding to Go transport\internet\grpc\config.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrpcConfig {
    #[serde(default)]
    pub service_name: String,
    #[serde(default)]
    pub multi_mode: bool,
    #[serde(default)]
    pub idle_timeout: u32,
    #[serde(default)]
    pub health_check_timeout: u32,
    #[serde(default)]
    pub permit_without_stream: bool,
    #[serde(default)]
    pub initial_windows_size: i32,
    #[serde(default)]
    pub user_agent: String,
}

impl GrpcConfig {
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            idle_timeout: 10,
            health_check_timeout: 20,
            ..Default::default()
        }
    }
}
