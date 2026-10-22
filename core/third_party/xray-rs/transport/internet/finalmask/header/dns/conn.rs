// Module: transport\internet\finalmask\header\dns\conn.rs
// 1:1 Rust implementation corresponding to Go transport\internet\finalmask\header\dns\conn.go

use super::DnsHeader;
use crate::common::errors::Result;

pub const DNS_PACKET_PORT: u16 = 53;

#[derive(Debug, Clone)]
pub struct DnsPacketConn {
    header: DnsHeader,
    cached_header: Vec<u8>,
}

impl DnsPacketConn {
    pub fn new(domain: impl Into<String>) -> Result<Self> {
        let domain_str = domain.into();
        let header = DnsHeader::new(&domain_str);
        let mut buf = vec![0u8; 512];
        let n = header.serialize(&mut buf)?;
        buf.truncate(n);
        Ok(Self {
            header,
            cached_header: buf,
        })
    }

    pub fn header(&self) -> &DnsHeader {
        &self.header
    }

    pub fn size(&self) -> usize {
        self.cached_header.len()
    }

    pub fn wrap_payload(&self, payload: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.cached_header.len() + payload.len());
        out.extend_from_slice(&self.cached_header);
        if out.len() >= 2 {
            let id: u16 = rand::random();
            out[0..2].copy_from_slice(&id.to_be_bytes());
        }
        out.extend_from_slice(payload);
        out
    }

    pub fn unwrap_payload<'a>(&self, packet: &'a [u8]) -> Option<&'a [u8]> {
        if packet.len() < self.size() {
            return None;
        }
        Some(&packet[self.size()..])
    }
}
