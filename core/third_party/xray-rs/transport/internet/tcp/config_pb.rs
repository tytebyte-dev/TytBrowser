// Module: transport\internet\tcp\config_pb.rs
// 1:1 Rust implementation corresponding to Go transport\internet\tcp\config.pb.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub header_settings: Option<String>,
    pub accept_proxy_protocol: bool,
}
