// Module: transport\internet\tcp\config.pb.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub header_settings: Option<String>,
    pub accept_proxy_protocol: bool,
}
