// Module: transport\internet\finalmask\salamander\conn.rs
// 1:1 Rust implementation corresponding to Go transport\internet\finalmask\salamander\conn.go

use super::config::SalamanderConfig;
use super::salamander::{SM_SALT_LEN, SalamanderObfuscator};
use crate::common::errors::{Error, Result};

pub struct SalamanderPacketConn {
    obfs: SalamanderObfuscator,
}

impl SalamanderPacketConn {
    pub fn new_client(config: &SalamanderConfig) -> Result<Self> {
        let obfs = SalamanderObfuscator::new(config.password.as_bytes().to_vec())?;
        Ok(Self { obfs })
    }

    pub fn new_server(config: &SalamanderConfig) -> Result<Self> {
        Self::new_client(config)
    }

    pub fn size(&self) -> usize {
        SM_SALT_LEN
    }

    pub fn mask(&self, payload: &[u8]) -> Vec<u8> {
        let mut out = vec![0u8; payload.len() + SM_SALT_LEN];
        self.obfs.obfuscate_slice(payload, &mut out);
        out
    }

    pub fn unmask(&self, packet: &[u8]) -> Result<Vec<u8>> {
        if packet.len() <= SM_SALT_LEN {
            return Err(Error::Protocol("salamander packet too short".into()));
        }
        let mut out = vec![0u8; packet.len() - SM_SALT_LEN];
        let n = self.obfs.deobfuscate_slice(packet, &mut out);
        if n == 0 {
            return Err(Error::Protocol("salamander deobfuscate failed".into()));
        }
        Ok(out)
    }
}
