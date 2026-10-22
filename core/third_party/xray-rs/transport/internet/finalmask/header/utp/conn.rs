// Module: transport\internet\finalmask\header\utp\conn.rs
// 1:1 Rust implementation corresponding to Go transport\internet\finalmask\header\utp\conn.go

use super::UtpHeader;

pub struct UtpPacketConn {
    header: UtpHeader,
}

impl UtpPacketConn {
    pub fn new() -> Self {
        Self {
            header: UtpHeader::new(),
        }
    }

    pub fn wrap_payload(&self, payload: &[u8]) -> Vec<u8> {
        let hdr = self.header.write_header();
        let mut out = Vec::with_capacity(4 + payload.len());
        out.extend_from_slice(&hdr);
        out.extend_from_slice(payload);
        out
    }

    pub fn unwrap_payload<'a>(&self, packet: &'a [u8]) -> Option<&'a [u8]> {
        if packet.len() < 4 {
            return None;
        }
        Some(&packet[4..])
    }
}

impl Default for UtpPacketConn {
    fn default() -> Self {
        Self::new()
    }
}
