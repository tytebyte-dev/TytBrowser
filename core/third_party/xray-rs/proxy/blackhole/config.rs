// Module: proxy\blackhole\config.rs
// 1:1 Rust implementation corresponding to Go proxy\blackhole\config.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ResponseType {
    #[default]
    None,
    Http403,
    Http500,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlackholeConfig {
    pub response: ResponseType,
}
