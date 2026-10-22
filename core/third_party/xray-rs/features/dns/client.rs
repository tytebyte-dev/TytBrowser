// Module: features\dns\client.rs
// 1:1 Rust implementation corresponding to Go features\dns\client.go

use crate::common::errors::Result;
use crate::features::feature::{Feature, TYPE_DNS_CLIENT};
use async_trait::async_trait;
use std::fmt;
use std::net::IpAddr;

pub const DEFAULT_TTL: u32 = 300;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IPOption {
    pub ipv4_enable: bool,
    pub ipv6_enable: bool,
    pub fake_enable: bool,
}

impl Default for IPOption {
    fn default() -> Self {
        Self {
            ipv4_enable: true,
            ipv6_enable: true,
            fake_enable: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RCodeError(pub u16);

impl fmt::Display for RCodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "rcode: {}", self.0)
    }
}

impl std::error::Error for RCodeError {}

#[async_trait]
pub trait DnsClient: Feature {
    async fn lookup_ip(&self, domain: &str) -> Result<Vec<IpAddr>> {
        self.lookup_ip_with_option(domain, IPOption::default())
            .await
            .map(|(ips, _)| ips)
    }

    async fn lookup_ip_with_option(
        &self,
        domain: &str,
        option: IPOption,
    ) -> Result<(Vec<IpAddr>, u32)>;
}

pub fn client_type() -> &'static str {
    TYPE_DNS_CLIENT
}
