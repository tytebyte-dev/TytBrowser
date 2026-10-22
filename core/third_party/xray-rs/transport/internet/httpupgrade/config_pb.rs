// Module: transport\internet\httpupgrade\config_pb.rs
// 1:1 Rust protobuf mapping corresponding to Go transport\internet\httpupgrade\config.pb.go

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub host: String,
    pub path: String,
    pub header: HashMap<String, String>,
    pub accept_proxy_protocol: bool,
}
