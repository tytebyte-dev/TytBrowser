// Module: proxy\shadowsocks_2022\inbound_multi.rs
// 1:1 Rust implementation corresponding to Go proxy\shadowsocks_2022\inbound_multi.go

use crate::common::errors::{Error, Result};
use crate::common::protocol::MemoryUser;
use std::sync::RwLock;

#[derive(Debug, Default)]
pub struct MultiUserInbound {
    users: RwLock<Vec<MemoryUser>>,
    method: String,
    psk: Vec<u8>,
}

impl MultiUserInbound {
    pub fn new(method: impl Into<String>, psk: Vec<u8>) -> Self {
        Self {
            users: RwLock::new(Vec::new()),
            method: method.into(),
            psk,
        }
    }

    pub fn add_user(&self, user: MemoryUser) -> Result<()> {
        let mut users = self.users.write().unwrap();
        if !user.email.is_empty() && users.iter().any(|u| u.email == user.email) {
            return Err(Error::Other(format!("User {} already exists", user.email)));
        }
        users.push(user);
        Ok(())
    }

    pub fn remove_user(&self, email: &str) -> Result<()> {
        if email.is_empty() {
            return Err(Error::Other("Email must not be empty".into()));
        }
        let mut users = self.users.write().unwrap();
        if let Some(pos) = users
            .iter()
            .position(|u| u.email.eq_ignore_ascii_case(email))
        {
            users.remove(pos);
            Ok(())
        } else {
            Err(Error::NotFound(format!("User {} not found", email)))
        }
    }

    pub fn get_user(&self, email: &str) -> Option<MemoryUser> {
        if email.is_empty() {
            return None;
        }
        let users = self.users.read().unwrap();
        users
            .iter()
            .find(|u| u.email.eq_ignore_ascii_case(email))
            .cloned()
    }

    pub fn get_users(&self) -> Vec<MemoryUser> {
        self.users.read().unwrap().clone()
    }

    pub fn get_users_count(&self) -> usize {
        self.users.read().unwrap().len()
    }

    pub fn method(&self) -> &str {
        &self.method
    }

    pub fn psk(&self) -> &[u8] {
        &self.psk
    }
}

pub use MultiUserInbound as Shadowsocks2022MultiInbound;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ss2022_multi_user_lifecycle() {
        let inbound = MultiUserInbound::new("2022-blake3-aes-128-gcm", vec![0u8; 16]);
        assert_eq!(inbound.get_users_count(), 0);

        let u1 = MemoryUser::new(uuid::Uuid::new_v4(), "alice@test.com", 1);
        inbound.add_user(u1.clone()).unwrap();
        assert_eq!(inbound.get_users_count(), 1);

        let retrieved = inbound.get_user("alice@test.com").unwrap();
        assert_eq!(retrieved.email, "alice@test.com");

        inbound.remove_user("alice@test.com").unwrap();
        assert_eq!(inbound.get_users_count(), 0);
    }
}
