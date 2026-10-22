// Module: infra\conf\common.rs
// 1:1 Rust implementation corresponding to Go infra\conf\common.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StringList(pub Vec<String>);

impl StringList {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, val: impl Into<String>) {
        self.0.push(val.into());
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
