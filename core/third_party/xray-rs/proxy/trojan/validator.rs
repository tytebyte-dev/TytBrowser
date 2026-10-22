use super::protocol::hash_password;
use std::collections::HashSet;
use std::sync::RwLock;

pub struct PasswordValidator {
    hashes: RwLock<HashSet<[u8; 56]>>,
}

impl PasswordValidator {
    pub fn new() -> Self {
        Self {
            hashes: RwLock::new(HashSet::new()),
        }
    }

    pub fn add_password(&self, password: &str) {
        let hash = hash_password(password);
        if let Ok(mut guard) = self.hashes.write() {
            guard.insert(hash);
        }
    }

    pub fn validate(&self, hash: &[u8; 56]) -> bool {
        if let Ok(guard) = self.hashes.read() {
            if guard.is_empty() {
                return true;
            }
            return guard.contains(hash);
        }
        false
    }
}

impl Default for PasswordValidator {
    fn default() -> Self {
        Self::new()
    }
}
