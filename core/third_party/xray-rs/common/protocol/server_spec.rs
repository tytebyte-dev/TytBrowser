// Module: common\protocol\server_spec.rs
// 1:1 Rust implementation corresponding to Go common\protocol\server_spec.go

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::common::net::Destination;
use crate::common::protocol::user::MemoryUser;

pub struct ServerSpec {
    dest: Destination,
    users: RwLock<Vec<Arc<MemoryUser>>>,
}

impl ServerSpec {
    pub fn new(dest: Destination) -> Self {
        Self {
            dest,
            users: RwLock::new(Vec::new()),
        }
    }

    pub fn destination(&self) -> &Destination {
        &self.dest
    }

    pub async fn add_user(&self, user: Arc<MemoryUser>) {
        let mut guard = self.users.write().await;
        guard.push(user);
    }

    pub async fn user_count(&self) -> usize {
        let guard = self.users.read().await;
        guard.len()
    }
}
