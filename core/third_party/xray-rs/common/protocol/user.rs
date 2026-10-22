use crate::common::protocol::id::Id;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityType {
    Unknown = 0,
    Legacy = 1,
    Auto = 2,
    Aes128Gcm = 3,
    ChaCha20Poly1305 = 4,
    None = 5,
    Zero = 6,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub level: u32,
}

impl User {
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            email: String::new(),
            level: 0,
        }
    }

    pub fn with_email(id: Uuid, email: impl Into<String>, level: u32) -> Self {
        Self {
            id,
            email: email.into(),
            level,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemoryUser {
    pub id: Id,
    pub email: String,
    pub level: u32,
}

impl MemoryUser {
    pub fn new(uuid: Uuid, email: impl Into<String>, level: u32) -> Self {
        Self {
            id: Id::new(uuid),
            email: email.into(),
            level,
        }
    }
}
