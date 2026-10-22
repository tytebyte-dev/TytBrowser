use std::net::IpAddr;
use std::time::{Duration, Instant};

pub fn fqdn(domain: &str) -> String {
    if domain.ends_with('.') {
        domain.to_string()
    } else {
        format!("{}.", domain)
    }
}

#[derive(Debug, Clone)]
pub struct IPRecord {
    pub req_id: u16,
    pub ips: Vec<IpAddr>,
    pub expire: Instant,
    pub rcode: u8,
}

impl IPRecord {
    pub fn new(req_id: u16, ips: Vec<IpAddr>, ttl: Duration, rcode: u8) -> Self {
        Self {
            req_id,
            ips,
            expire: Instant::now() + ttl,
            rcode,
        }
    }

    pub fn is_expired(&self) -> bool {
        Instant::now() >= self.expire
    }

    pub fn get_ips(&self) -> Option<&[IpAddr]> {
        if self.is_expired() || self.rcode != 0 || self.ips.is_empty() {
            None
        } else {
            Some(&self.ips)
        }
    }
}
