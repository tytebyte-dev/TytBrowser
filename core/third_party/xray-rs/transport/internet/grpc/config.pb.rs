// Module: transport\internet\grpc\config.pb.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub service_name: String,
    pub multi_mode: bool,
    pub idle_timeout: i32,
    pub health_check_timeout: i32,
    pub permit_without_stream: bool,
    pub initial_windows_size: i32,
    pub user_agent: String,
}
