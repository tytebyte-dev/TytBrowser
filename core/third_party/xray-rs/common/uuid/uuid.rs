// Module: common\uuid\uuid.rs
// 1:1 Rust implementation corresponding to Go common\uuid\uuid.go

use crate::common::errors::{Error, Result};
use rand::Rng;
use sha1::{Digest, Sha1};
use std::fmt;

/// UUID is a 16-byte identifier corresponding to Go `uuid.UUID`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UUID(pub [u8; 16]);

impl UUID {
    /// Creates a UUID with random value (v4).
    pub fn new() -> Self {
        let mut bytes = [0u8; 16];
        rand::thread_rng().fill(&mut bytes);
        bytes[6] = (bytes[6] & 0x0f) | (4 << 4);
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        UUID(bytes)
    }

    /// Bytes returns the bytes slice of this UUID.
    pub fn bytes(&self) -> &[u8; 16] {
        &self.0
    }

    pub fn to_bytes(&self) -> [u8; 16] {
        self.0
    }

    /// Equals returns true if this UUID equals another UUID by value.
    pub fn equals(&self, another: Option<&UUID>) -> bool {
        match another {
            Some(a) => self.0 == a.0,
            None => false,
        }
    }

    /// String returns the canonical 8-4-4-4-12 hex string representation.
    pub fn to_string_formatted(&self) -> String {
        format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            self.0[0],
            self.0[1],
            self.0[2],
            self.0[3],
            self.0[4],
            self.0[5],
            self.0[6],
            self.0[7],
            self.0[8],
            self.0[9],
            self.0[10],
            self.0[11],
            self.0[12],
            self.0[13],
            self.0[14],
            self.0[15]
        )
    }
}

/// ParseBytes converts a UUID in byte form to object.
pub fn parse_bytes(b: &[u8]) -> Result<UUID> {
    if b.len() != 16 {
        return Err(Error::Protocol(
            "invalid UUID: slice must be 16 bytes".into(),
        ));
    }
    let mut out = [0u8; 16];
    out.copy_from_slice(b);
    Ok(UUID(out))
}

/// ParseString converts a UUID in string form to object.
/// For inputs of length 1..30, it derives a deterministic v5-style UUID via SHA1 matching Go.
pub fn parse_string(str: &str) -> Result<UUID> {
    let text = str.as_bytes();
    let l = text.len();
    if !(32..=36).contains(&l) {
        if l == 0 || l > 30 {
            return Err(Error::Protocol(format!("invalid UUID: {}", str)));
        }
        let mut hasher = Sha1::new();
        hasher.update([0u8; 16]);
        hasher.update(text);
        let result = hasher.finalize();
        let mut u = [0u8; 16];
        u.copy_from_slice(&result[..16]);
        u[6] = (u[6] & 0x0f) | (5 << 4);
        u[8] = (u[8] & 0x3f) | 0x80;
        return Ok(UUID(u));
    }

    let cleaned: String = str.chars().filter(|&c| c != '-').collect();
    if cleaned.len() != 32 {
        return Err(Error::Protocol(format!("invalid UUID: {}", str)));
    }

    let mut out = [0u8; 16];
    for i in 0..16 {
        out[i] = u8::from_str_radix(&cleaned[i * 2..i * 2 + 2], 16)
            .map_err(|e| Error::Protocol(format!("invalid UUID hex: {}", e)))?;
    }
    Ok(UUID(out))
}

impl fmt::Display for UUID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_formatted())
    }
}

impl From<uuid::Uuid> for UUID {
    fn from(u: uuid::Uuid) -> Self {
        UUID(*u.as_bytes())
    }
}

impl From<UUID> for uuid::Uuid {
    fn from(u: UUID) -> Self {
        uuid::Uuid::from_bytes(u.0)
    }
}

pub fn new_uuid() -> uuid::Uuid {
    uuid::Uuid::new_v4()
}

pub fn parse_uuid(s: &str) -> Option<uuid::Uuid> {
    uuid::Uuid::parse_str(s).ok()
}
