use crate::common::errors::{Error, Result};
use crate::common::protocol::User;
use crate::common::uuid::process_uuid;
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

pub struct MemoryValidator {
    users: RwLock<HashMap<[u8; 16], User>>,
}

impl MemoryValidator {
    pub fn new() -> Self {
        Self {
            users: RwLock::new(HashMap::new()),
        }
    }

    pub fn add(&self, user: User) -> Result<()> {
        let normalized = process_uuid(*user.id.as_bytes());
        let mut guard = self
            .users
            .write()
            .map_err(|e| Error::Other(format!("Lock poisoned: {}", e)))?;
        guard.insert(normalized, user);
        Ok(())
    }

    pub fn get(&self, id: &Uuid) -> Option<User> {
        let normalized = process_uuid(*id.as_bytes());
        let guard = self.users.read().ok()?;
        guard.get(&normalized).cloned()
    }

    pub fn count(&self) -> usize {
        self.users.read().map(|g| g.len()).unwrap_or(0)
    }
}

impl Default for MemoryValidator {
    fn default() -> Self {
        Self::new()
    }
}
