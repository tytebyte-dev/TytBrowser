// Module: transport\internet\finalmask\header\custom\udp.rs
// 1:1 Rust implementation corresponding to Go transport\internet\finalmask\header\custom\udp.go

use rand::Rng;

use super::config::{UDPConfig, UDPItem};
use crate::common::errors::{Error, Result};

pub const UDP_CUSTOM_MAGIC: &[u8] = b"CUSTOM-UDP";

#[derive(Debug, Clone)]
pub struct UdpCustomClient {
    client: Vec<UDPItem>,
    server: Vec<UDPItem>,
    template: Vec<u8>,
}

impl UdpCustomClient {
    pub fn new(config: &UDPConfig) -> Self {
        let mut template = Vec::new();
        for item in &config.client {
            if item.rand > 0 {
                template.resize(template.len() + item.rand as usize, 0);
            } else {
                template.extend_from_slice(&item.packet);
            }
        }
        Self {
            client: config.client.clone(),
            server: config.server.clone(),
            template,
        }
    }

    pub fn size(&self) -> usize {
        self.template.len()
    }

    pub fn serialize(&self, buf: &mut [u8]) {
        let mut rng = rand::thread_rng();
        let mut out = self.template.clone();
        let mut idx = 0;
        for item in &self.client {
            if item.rand > 0 {
                let r_min = (item.rand_min as u8).min(item.rand_max as u8);
                let r_max = (item.rand_max as u8).max(r_min);
                for _ in 0..item.rand {
                    out[idx] = if r_min == r_max {
                        r_min
                    } else {
                        rng.gen_range(r_min..=r_max)
                    };
                    idx += 1;
                }
            } else {
                idx += item.packet.len();
            }
        }
        let copy_len = buf.len().min(out.len());
        buf[..copy_len].copy_from_slice(&out[..copy_len]);
    }

    pub fn matches(&self, packet: &[u8]) -> bool {
        let mut data = packet;
        for item in &self.server {
            let length = (item.rand as usize).max(item.packet.len());
            if data.len() < length {
                return false;
            }
            if !item.packet.is_empty() && &data[..item.packet.len()] != item.packet.as_slice() {
                return false;
            }
            data = &data[length..];
        }
        true
    }

    pub fn wrap_payload(&self, payload: &[u8]) -> Vec<u8> {
        let mut out = vec![0u8; self.size() + payload.len()];
        self.serialize(&mut out[..self.size()]);
        out[self.size()..].copy_from_slice(payload);
        out
    }

    pub fn unwrap_payload<'a>(&self, packet: &'a [u8]) -> Result<&'a [u8]> {
        if !self.matches(packet) {
            return Err(Error::Protocol("UDP custom header mismatch".into()));
        }
        let mut consumed = 0;
        for item in &self.server {
            consumed += (item.rand as usize).max(item.packet.len());
        }
        if packet.len() < consumed {
            return Err(Error::Protocol("UDP packet too short".into()));
        }
        Ok(&packet[consumed..])
    }
}

#[derive(Debug, Clone)]
pub struct UdpCustomServer {
    client: Vec<UDPItem>,
    server: Vec<UDPItem>,
    template: Vec<u8>,
}

impl UdpCustomServer {
    pub fn new(config: &UDPConfig) -> Self {
        let mut template = Vec::new();
        for item in &config.server {
            if item.rand > 0 {
                template.resize(template.len() + item.rand as usize, 0);
            } else {
                template.extend_from_slice(&item.packet);
            }
        }
        Self {
            client: config.client.clone(),
            server: config.server.clone(),
            template,
        }
    }

    pub fn size(&self) -> usize {
        self.template.len()
    }

    pub fn serialize(&self, buf: &mut [u8]) {
        let mut rng = rand::thread_rng();
        let mut out = self.template.clone();
        let mut idx = 0;
        for item in &self.server {
            if item.rand > 0 {
                let r_min = (item.rand_min as u8).min(item.rand_max as u8);
                let r_max = (item.rand_max as u8).max(r_min);
                for _ in 0..item.rand {
                    out[idx] = if r_min == r_max {
                        r_min
                    } else {
                        rng.gen_range(r_min..=r_max)
                    };
                    idx += 1;
                }
            } else {
                idx += item.packet.len();
            }
        }
        let copy_len = buf.len().min(out.len());
        buf[..copy_len].copy_from_slice(&out[..copy_len]);
    }

    pub fn matches(&self, packet: &[u8]) -> bool {
        let mut data = packet;
        for item in &self.client {
            let length = (item.rand as usize).max(item.packet.len());
            if data.len() < length {
                return false;
            }
            if !item.packet.is_empty() && &data[..item.packet.len()] != item.packet.as_slice() {
                return false;
            }
            data = &data[length..];
        }
        true
    }

    pub fn wrap_payload(&self, payload: &[u8]) -> Vec<u8> {
        let mut out = vec![0u8; self.size() + payload.len()];
        self.serialize(&mut out[..self.size()]);
        out[self.size()..].copy_from_slice(payload);
        out
    }

    pub fn unwrap_payload<'a>(&self, packet: &'a [u8]) -> Result<&'a [u8]> {
        if !self.matches(packet) {
            return Err(Error::Protocol("UDP custom header mismatch".into()));
        }
        let mut consumed = 0;
        for item in &self.client {
            consumed += (item.rand as usize).max(item.packet.len());
        }
        if packet.len() < consumed {
            return Err(Error::Protocol("UDP packet too short".into()));
        }
        Ok(&packet[consumed..])
    }
}
