// Module: proxy\dokodemo\config.pb.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub address: Option<String>,
    pub port: u32,
    pub network_list: Vec<i32>,
    pub timeout: u32,
    pub follow_redirect: bool,
}
