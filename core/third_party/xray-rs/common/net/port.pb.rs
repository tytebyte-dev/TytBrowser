// Module: common\net\port.pb.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortRange {
    pub from: u32,
    pub to: u32,
}
