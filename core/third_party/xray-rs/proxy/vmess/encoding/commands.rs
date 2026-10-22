// Module: proxy\vmess\encoding\commands.rs
// 1:1 Rust implementation corresponding to Go proxy\vmess\encoding\commands.go

use super::auth::authenticate;
use crate::common::errors::{Error, Result};
use uuid::Uuid;

pub const CMD_TCP: u8 = 1;
pub const CMD_UDP: u8 = 2;
pub const CMD_MUX: u8 = 3;
pub const CMD_SWITCH_ACCOUNT: u8 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSwitchAccount {
    pub host: String,
    pub port: u16,
    pub id: Uuid,
    pub alter_id: u16,
    pub level: u32,
    pub valid_min: u8,
}

impl CommandSwitchAccount {
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        let host_bytes = self.host.as_bytes();
        buf.push(host_bytes.len() as u8);
        buf.extend_from_slice(host_bytes);
        buf.extend_from_slice(&self.port.to_be_bytes());
        buf.extend_from_slice(self.id.as_bytes());
        buf.extend_from_slice(&self.alter_id.to_be_bytes());
        buf.extend_from_slice(&self.level.to_be_bytes());
        buf.push(self.valid_min);
        buf
    }

    pub fn decode(b: &[u8]) -> Result<Self> {
        if b.is_empty() {
            return Err(Error::Protocol("Empty switch account data".into()));
        }
        let host_len = b[0] as usize;
        let mut idx = 1;
        if b.len() < idx + host_len + 2 + 16 + 2 + 4 + 1 {
            return Err(Error::Protocol(
                "Insufficient length for CommandSwitchAccount".into(),
            ));
        }
        let host = String::from_utf8(b[idx..idx + host_len].to_vec())
            .map_err(|e| Error::Protocol(format!("Invalid host in CommandSwitchAccount: {}", e)))?;
        idx += host_len;

        let port = u16::from_be_bytes([b[idx], b[idx + 1]]);
        idx += 2;

        let id = Uuid::from_slice(&b[idx..idx + 16])
            .map_err(|e| Error::Protocol(format!("Invalid UUID: {}", e)))?;
        idx += 16;

        let alter_id = u16::from_be_bytes([b[idx], b[idx + 1]]);
        idx += 2;

        let level = u32::from_be_bytes([b[idx], b[idx + 1], b[idx + 2], b[idx + 3]]);
        idx += 4;

        let valid_min = b[idx];

        Ok(Self {
            host,
            port,
            id,
            alter_id,
            level,
            valid_min,
        })
    }
}

pub fn marshal_command(cmd_id: u8, payload: &[u8]) -> Result<Vec<u8>> {
    let auth = authenticate(payload);
    let total_len = payload.len() + 4;
    if total_len > 255 {
        return Err(Error::Protocol("Command too large".into()));
    }

    let mut out = Vec::with_capacity(2 + 4 + payload.len());
    out.push(cmd_id);
    out.push(total_len as u8);
    out.extend_from_slice(&auth.to_be_bytes());
    out.extend_from_slice(payload);
    Ok(out)
}

pub fn unmarshal_command(data: &[u8]) -> Result<(u8, Vec<u8>)> {
    if data.len() < 6 {
        return Err(Error::Protocol(
            "Insufficient length for command frame".into(),
        ));
    }
    let cmd_id = data[0];
    let total_len = data[1] as usize;
    if data.len() < 2 + total_len {
        return Err(Error::Protocol("Truncated command payload".into()));
    }
    let expected_auth = u32::from_be_bytes([data[2], data[3], data[4], data[5]]);
    let payload = &data[6..2 + total_len];
    let actual_auth = authenticate(payload);
    if expected_auth != actual_auth {
        return Err(Error::AuthFailed("Invalid command auth checksum".into()));
    }
    Ok((cmd_id, payload.to_vec()))
}
