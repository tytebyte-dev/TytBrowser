// Module: transport\internet\websocket\config.pb.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub path: String,
    pub host: String,
    pub max_early_data: i32,
    pub use_browser_forwarding: bool,
    pub early_data_header_name: String,
}
