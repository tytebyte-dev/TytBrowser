use crate::app::dns::fakedns::FakeDnsHolder;
use std::net::IpAddr;
use std::sync::Arc;

pub struct FakeDnsSniffer {
    holder: Arc<FakeDnsHolder>,
}

impl FakeDnsSniffer {
    pub fn new(holder: Arc<FakeDnsHolder>) -> Self {
        Self { holder }
    }

    pub fn sniff_domain(&self, ip: &IpAddr) -> Option<String> {
        match ip {
            IpAddr::V4(v4) => self.holder.get_domain_for_fake_ip(v4),
            _ => None,
        }
    }

    pub fn is_in_pool(&self, ip: &IpAddr) -> bool {
        self.holder.is_fake_ip(ip)
    }
}
