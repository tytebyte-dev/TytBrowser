pub mod uuid;

#[cfg(test)]
pub mod uuid_test;

use crate::common::errors::{Error, Result};
use ::uuid::Uuid;
pub use uuid::{UUID, new_uuid, parse_bytes, parse_string, parse_uuid};

pub fn process_uuid(uuid_bytes: [u8; 16]) -> [u8; 16] {
    uuid_bytes
}

pub fn parse_str(s: &str) -> Result<Uuid> {
    Uuid::parse_str(s).map_err(|e| Error::Protocol(format!("Invalid UUID: {}", e)))
}
