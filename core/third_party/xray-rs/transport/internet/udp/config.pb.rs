// Module: transport\internet\udp\config.pb.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub mtu: u32,
    pub ttl: u32,
}
