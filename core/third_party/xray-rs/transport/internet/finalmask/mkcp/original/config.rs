// Module: transport\internet\finalmask\mkcp\original\config.rs
// 1:1 Rust implementation corresponding to Go transport\internet\finalmask\mkcp\original\config.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OriginalConfig {
    pub xor_mask: u8,
}

impl OriginalConfig {
    pub fn mask(&self) -> u8 {
        self.xor_mask
    }
}

pub type OriginalMkcpConfig = OriginalConfig;
