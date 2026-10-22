// Module: transport\internet\finalmask\salamander\config.rs
// 1:1 Rust implementation corresponding to Go transport\internet\finalmask\salamander\config.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SalamanderConfig {
    pub password: String,
}

impl SalamanderConfig {
    pub fn new(password: impl Into<String>) -> Self {
        Self {
            password: password.into(),
        }
    }
}
