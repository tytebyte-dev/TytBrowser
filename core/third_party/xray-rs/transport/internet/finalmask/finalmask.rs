// Module: transport\internet\finalmask\finalmask.rs
// 1:1 Rust implementation corresponding to Go transport\internet\finalmask\finalmask.go

use super::header::custom::udp::{UdpCustomClient, UdpCustomServer};
use super::header::dns::conn::DnsPacketConn;
use super::header::dtls::conn::DtlsPacketConn;
use super::header::srtp::conn::SrtpPacketConn;
use super::header::utp::conn::UtpPacketConn;
use super::header::wechat::conn::WeChatPacketConn;
use super::header::wireguard::conn::WireguardPacketConn;
use crate::common::errors::{Error, Result};
use std::sync::Arc;

pub const FINALMASK_VERSION: u32 = 1;
pub const UDP_SIZE: usize = 4096;

/// HeaderMask defines a packet-level obfuscation header that prepends a fixed or dynamic header.
pub trait HeaderMask: Send + Sync {
    fn size(&self) -> usize;
    fn wrap_packet(&mut self, payload: &[u8]) -> Vec<u8>;
    fn unwrap_packet<'a>(&self, packet: &'a [u8]) -> Option<&'a [u8]>;
}

impl HeaderMask for DtlsPacketConn {
    fn size(&self) -> usize {
        self.size()
    }
    fn wrap_packet(&mut self, payload: &[u8]) -> Vec<u8> {
        self.wrap_payload(payload)
    }
    fn unwrap_packet<'a>(&self, packet: &'a [u8]) -> Option<&'a [u8]> {
        self.unwrap_payload(packet)
    }
}

impl HeaderMask for WeChatPacketConn {
    fn size(&self) -> usize {
        self.size()
    }
    fn wrap_packet(&mut self, payload: &[u8]) -> Vec<u8> {
        self.wrap_payload(payload)
    }
    fn unwrap_packet<'a>(&self, packet: &'a [u8]) -> Option<&'a [u8]> {
        self.unwrap_payload(packet)
    }
}

impl HeaderMask for DnsPacketConn {
    fn size(&self) -> usize {
        self.size()
    }
    fn wrap_packet(&mut self, payload: &[u8]) -> Vec<u8> {
        self.wrap_payload(payload)
    }
    fn unwrap_packet<'a>(&self, packet: &'a [u8]) -> Option<&'a [u8]> {
        self.unwrap_payload(packet)
    }
}

impl HeaderMask for SrtpPacketConn {
    fn size(&self) -> usize {
        4
    }
    fn wrap_packet(&mut self, payload: &[u8]) -> Vec<u8> {
        self.wrap_payload(payload)
    }
    fn unwrap_packet<'a>(&self, packet: &'a [u8]) -> Option<&'a [u8]> {
        self.unwrap_payload(packet)
    }
}

impl HeaderMask for UtpPacketConn {
    fn size(&self) -> usize {
        4
    }
    fn wrap_packet(&mut self, payload: &[u8]) -> Vec<u8> {
        self.wrap_payload(payload)
    }
    fn unwrap_packet<'a>(&self, packet: &'a [u8]) -> Option<&'a [u8]> {
        self.unwrap_payload(packet)
    }
}

impl HeaderMask for WireguardPacketConn {
    fn size(&self) -> usize {
        4
    }
    fn wrap_packet(&mut self, payload: &[u8]) -> Vec<u8> {
        self.wrap_payload(payload)
    }
    fn unwrap_packet<'a>(&self, packet: &'a [u8]) -> Option<&'a [u8]> {
        self.unwrap_payload(packet)
    }
}

impl HeaderMask for UdpCustomClient {
    fn size(&self) -> usize {
        self.size()
    }
    fn wrap_packet(&mut self, payload: &[u8]) -> Vec<u8> {
        self.wrap_payload(payload)
    }
    fn unwrap_packet<'a>(&self, packet: &'a [u8]) -> Option<&'a [u8]> {
        self.unwrap_payload(packet).ok()
    }
}

impl HeaderMask for UdpCustomServer {
    fn size(&self) -> usize {
        self.size()
    }
    fn wrap_packet(&mut self, payload: &[u8]) -> Vec<u8> {
        self.wrap_payload(payload)
    }
    fn unwrap_packet<'a>(&self, packet: &'a [u8]) -> Option<&'a [u8]> {
        self.unwrap_payload(packet).ok()
    }
}

/// HeaderManager manages a chained list of header masks (corresponding to headerManagerConn in Go)
pub struct HeaderManager {
    headers: Vec<Box<dyn HeaderMask>>,
}

impl HeaderManager {
    pub fn new(headers: Vec<Box<dyn HeaderMask>>) -> Self {
        Self { headers }
    }

    pub fn total_size(&self) -> usize {
        self.headers.iter().map(|h| h.size()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.headers.is_empty()
    }

    pub fn len(&self) -> usize {
        self.headers.len()
    }

    /// Wraps outgoing payload with all headers in reverse order (outer-to-inner during prepending)
    pub fn wrap_outgoing(&mut self, payload: &[u8]) -> Result<Vec<u8>> {
        if self.total_size() + payload.len() > UDP_SIZE {
            return Err(Error::Protocol(
                "packet size exceeds finalmask UDP limit".into(),
            ));
        }
        let mut buf = payload.to_vec();
        for h in self.headers.iter_mut().rev() {
            buf = h.wrap_packet(&buf);
        }
        Ok(buf)
    }

    /// Strips each header in forward order
    pub fn unwrap_incoming<'a>(&self, packet: &'a [u8]) -> Result<&'a [u8]> {
        let sum = self.total_size();
        if packet.len() < sum {
            return Err(Error::Protocol("packet too short for header stack".into()));
        }
        let mut current = packet;
        for h in &self.headers {
            current = h
                .unwrap_packet(current)
                .ok_or_else(|| Error::Protocol("finalmask header validation failed".into()))?;
        }
        Ok(current)
    }
}

/// UdpMask defines general UDP transformations
pub trait UdpMask: Send + Sync {
    fn wrap_client(&mut self, payload: &[u8]) -> Result<Vec<u8>>;
    fn unwrap_client<'a>(&self, packet: &'a [u8]) -> Result<&'a [u8]>;
    fn wrap_server(&mut self, payload: &[u8]) -> Result<Vec<u8>>;
    fn unwrap_server<'a>(&self, packet: &'a [u8]) -> Result<&'a [u8]>;
}

/// UdpmaskManager manages multiple UDP obfuscators and header wrappers
pub struct UdpmaskManager {
    headers: Option<HeaderManager>,
    masks: Vec<Box<dyn UdpMask>>,
}

impl UdpmaskManager {
    pub fn new(headers: Option<HeaderManager>, masks: Vec<Box<dyn UdpMask>>) -> Self {
        Self { headers, masks }
    }

    pub fn wrap_client(&mut self, payload: &[u8]) -> Result<Vec<u8>> {
        let mut current = payload.to_vec();
        for mask in &mut self.masks {
            current = mask.wrap_client(&current)?;
        }
        if let Some(ref mut hm) = self.headers {
            current = hm.wrap_outgoing(&current)?;
        }
        Ok(current)
    }

    pub fn unwrap_client<'a>(&'a self, packet: &'a [u8]) -> Result<&'a [u8]> {
        let mut current = packet;
        if let Some(ref hm) = self.headers {
            current = hm.unwrap_incoming(current)?;
        }
        for mask in &self.masks {
            current = mask.unwrap_client(current)?;
        }
        Ok(current)
    }

    pub fn wrap_server(&mut self, payload: &[u8]) -> Result<Vec<u8>> {
        let mut current = payload.to_vec();
        for mask in &mut self.masks {
            current = mask.wrap_server(&current)?;
        }
        if let Some(ref mut hm) = self.headers {
            current = hm.wrap_outgoing(&current)?;
        }
        Ok(current)
    }

    pub fn unwrap_server<'a>(&'a self, packet: &'a [u8]) -> Result<&'a [u8]> {
        let mut current = packet;
        if let Some(ref hm) = self.headers {
            current = hm.unwrap_incoming(current)?;
        }
        for mask in &self.masks {
            current = mask.unwrap_server(current)?;
        }
        Ok(current)
    }
}

/// TcpMask trait for connection level stream obfuscation
pub trait TcpMask: Send + Sync {
    fn name(&self) -> &str;
}

/// TcpmaskManager manages multiple TCP connection masks
pub struct TcpmaskManager {
    masks: Vec<Arc<dyn TcpMask>>,
}

impl TcpmaskManager {
    pub fn new(masks: Vec<Arc<dyn TcpMask>>) -> Self {
        Self { masks }
    }

    pub fn len(&self) -> usize {
        self.masks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.masks.is_empty()
    }
}

/// Trait for inspecting connection unwrapping and splicing capability
pub trait TcpMaskConn {
    fn splice(&self) -> bool {
        true
    }
}

/// Helper to check if a TCP stream is unwrappable
pub fn unwrap_tcp_mask<T: TcpMaskConn>(conn: &T) -> bool {
    conn.splice()
}
