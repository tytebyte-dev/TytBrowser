// Module: transport\internet\finalmask\mkcp\original\config.pb.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub xor_mask: u32,
}
