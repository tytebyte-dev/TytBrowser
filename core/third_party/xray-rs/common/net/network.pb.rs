// Module: common\net\network.pb.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Network {
    Unknown = 0,
    RawTcp = 1,
    Tcp = 2,
    Udp = 3,
    Unix = 4,
}

impl Default for Network {
    fn default() -> Self {
        Self::Tcp
    }
}
