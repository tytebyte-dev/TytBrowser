// Module: features\dns\fakedns.rs
// 1:1 Rust implementation corresponding to Go features\dns\fakedns.go

use crate::features::feature::{Feature, TYPE_FAKE_DNS};
use std::net::IpAddr;

pub const FAKE_IPV4_POOL: &str = "198.18.0.0/15";
pub const FAKE_IPV6_POOL: &str = "fc00::/18";

pub trait FakeDnsEngine: Feature {
    fn get_fake_ip_for_domain(&self, domain: &str) -> Vec<IpAddr>;
    fn get_domain_from_fake_dns(&self, ip: &IpAddr) -> Option<String>;
    fn is_ip_in_ip_pool(&self, ip: &IpAddr) -> bool;
    fn get_fake_ip_for_domain_with_options(
        &self,
        domain: &str,
        ipv4: bool,
        ipv6: bool,
    ) -> Vec<IpAddr> {
        let ips = self.get_fake_ip_for_domain(domain);
        ips.into_iter()
            .filter(|ip| match ip {
                IpAddr::V4(_) => ipv4,
                IpAddr::V6(_) => ipv6,
            })
            .collect()
    }
}

pub trait FakeDnsFeature: FakeDnsEngine {
    fn query_ip(&self, domain: &str) -> Option<IpAddr> {
        self.get_fake_ip_for_domain(domain).into_iter().next()
    }

    fn query_domain(&self, ip: &IpAddr) -> Option<String> {
        self.get_domain_from_fake_dns(ip)
    }
}

impl<T: FakeDnsEngine + ?Sized> FakeDnsFeature for T {}

pub fn fake_dns_type() -> &'static str {
    TYPE_FAKE_DNS
}
