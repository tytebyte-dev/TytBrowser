use crate::common::errors::{Error, Result};
use crate::common::protocol::User;
use crate::common::uuid::process_uuid;
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

pub trait Validator: Send + Sync {
    fn get(&self, id: &Uuid) -> Option<User>;
    fn add(&self, user: User) -> Result<()>;
    fn del(&self, email: &str) -> Result<()>;
    fn get_count(&self) -> usize;
}

pub struct MemoryValidator {
    users: RwLock<HashMap<[u8; 16], User>>,
    emails: RwLock<HashMap<String, [u8; 16]>>,
}

impl MemoryValidator {
    pub fn new() -> Self {
        Self {
            users: RwLock::new(HashMap::new()),
            emails: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for MemoryValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl Validator for MemoryValidator {
    fn get(&self, id: &Uuid) -> Option<User> {
        let normalized = process_uuid(*id.as_bytes());
        let guard = self.users.read().ok()?;
        guard.get(&normalized).cloned()
    }

    fn add(&self, user: User) -> Result<()> {
        let normalized = process_uuid(*user.id.as_bytes());
        let mut users_guard = self
            .users
            .write()
            .map_err(|e| Error::Other(format!("Lock poisoned: {}", e)))?;
        let mut emails_guard = self
            .emails
            .write()
            .map_err(|e| Error::Other(format!("Lock poisoned: {}", e)))?;

        if !user.email.is_empty() {
            let email_lower = user.email.to_lowercase();
            if emails_guard.contains_key(&email_lower) {
                return Err(Error::Config(format!(
                    "User email '{}' already exists",
                    user.email
                )));
            }
            emails_guard.insert(email_lower, normalized);
        }

        users_guard.insert(normalized, user);
        Ok(())
    }

    fn del(&self, email: &str) -> Result<()> {
        if email.is_empty() {
            return Err(Error::Config("Email cannot be empty".into()));
        }
        let email_lower = email.to_lowercase();
        let mut emails_guard = self
            .emails
            .write()
            .map_err(|e| Error::Other(format!("Lock poisoned: {}", e)))?;
        let mut users_guard = self
            .users
            .write()
            .map_err(|e| Error::Other(format!("Lock poisoned: {}", e)))?;

        let id = emails_guard
            .remove(&email_lower)
            .ok_or_else(|| Error::NotFound(format!("User '{}' not found", email)))?;
        users_guard.remove(&id);
        Ok(())
    }

    fn get_count(&self) -> usize {
        self.users.read().map(|g| g.len()).unwrap_or(0)
    }
}
