// Module: transport\internet\finalmask\header\dtls\conn.rs
// 1:1 Rust implementation corresponding to Go transport\internet\finalmask\header\dtls\conn.go

use super::DtlsHeader;

pub const DTLS_CONTENT_TYPE_HANDSHAKE: u8 = 22;
pub const DTLS_CONTENT_TYPE_APPLICATION_DATA: u8 = 23;

#[derive(Debug, Clone)]
pub struct DtlsPacketConn {
    header: DtlsHeader,
}

impl DtlsPacketConn {
    pub fn new() -> Self {
        Self {
            header: DtlsHeader::new(),
        }
    }

    pub fn size(&self) -> usize {
        self.header.size()
    }

    pub fn wrap_payload(&mut self, payload: &[u8]) -> Vec<u8> {
        let hdr = self.header.write_header();
        let mut out = Vec::with_capacity(hdr.len() + payload.len());
        out.extend_from_slice(&hdr);
        out.extend_from_slice(payload);
        out
    }

    pub fn unwrap_payload<'a>(&self, packet: &'a [u8]) -> Option<&'a [u8]> {
        if packet.len() < self.size() {
            return None;
        }
        Some(&packet[self.size()..])
    }

    pub fn serialize(&mut self, buf: &mut [u8]) {
        self.header.serialize(buf);
    }
}

impl Default for DtlsPacketConn {
    fn default() -> Self {
        Self::new()
    }
}
