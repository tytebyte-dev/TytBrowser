// Module: common\protocol\context.rs
// 1:1 Rust implementation corresponding to Go common\protocol\context.go

use crate::common::protocol::user::MemoryUser;
use std::sync::Arc;

#[derive(Debug, Clone, Default)]
pub struct ProtocolContext {
    pub user: Option<Arc<MemoryUser>>,
}

impl ProtocolContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_user(user: Arc<MemoryUser>) -> Self {
        Self { user: Some(user) }
    }

    pub fn user(&self) -> Option<&Arc<MemoryUser>> {
        self.user.as_ref()
    }
}
