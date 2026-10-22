// Module: transport\internet\finalmask\header\srtp\conn.rs
// 1:1 Rust implementation corresponding to Go transport\internet\finalmask\header\srtp\conn.go

use super::SrtpHeader;

pub struct SrtpPacketConn {
    header: SrtpHeader,
}

impl SrtpPacketConn {
    pub fn new() -> Self {
        Self {
            header: SrtpHeader::new(),
        }
    }

    pub fn wrap_payload(&mut self, payload: &[u8]) -> Vec<u8> {
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

impl Default for SrtpPacketConn {
    fn default() -> Self {
        Self::new()
    }
}
