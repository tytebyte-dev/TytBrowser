// Module: proxy\freedom\config.pb.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub domain_strategy: i32,
    pub timeout: u32,
    pub destination_override: Option<String>,
    pub user_level: u32,
}
