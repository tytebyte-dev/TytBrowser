// Module: common\protocol\quic\sniff.rs
// 1:1 Rust implementation corresponding to Go common\protocol\quic\sniff.go

use crate::common::errors::{Error, Result};
use crate::common::protocol::tls::sniff::read_client_hello;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuicSniffHeader {
    pub domain: String,
}

impl QuicSniffHeader {
    pub fn protocol(&self) -> &'static str {
        "quic"
    }

    pub fn domain(&self) -> &str {
        &self.domain
    }
}

pub fn sniff_quic(b: &[u8]) -> Result<QuicSniffHeader> {
    if b.is_empty() {
        return Err(Error::Protocol("empty quic packet".into()));
    }

    let type_byte = b[0];
    let is_long_header = (type_byte & 0x80) > 0;
    if !is_long_header || (type_byte & 0x40) == 0 {
        return Err(Error::Protocol("not quic initial packet".into()));
    }

    // Try finding TLS client hello within packet payload if unencrypted or after crypto frame
    if let Ok(domain) = read_client_hello(b) {
        return Ok(QuicSniffHeader { domain });
    }

    // Scan for SNI patterns in initial frame
    for i in 0..b.len().saturating_sub(4) {
        if b[i] == 0x00 && b[i + 1] == 0x00 {
            let rem = &b[i + 2..];
            if let Ok(domain) = read_client_hello(rem) {
                return Ok(QuicSniffHeader { domain });
            }
        }
    }

    Err(Error::Protocol("not quic or need more data".into()))
}
