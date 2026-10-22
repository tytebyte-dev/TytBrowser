// Module: proxy\tun\config.pb.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub mtu: u32,
    pub auto_route: bool,
    pub strict_route: bool,
}
