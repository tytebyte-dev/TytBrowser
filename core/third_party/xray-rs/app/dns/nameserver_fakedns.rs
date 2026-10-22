// Module: app\dns\nameserver_fakedns.rs
// 1:1 Rust implementation corresponding to Go app\dns\nameserver_fakedns.go

use async_trait::async_trait;
use std::net::IpAddr;
use std::sync::Arc;

use super::fakedns::FakeDnsHolder;
use super::nameserver::NameServer;
use crate::common::errors::Result;

pub struct FakeDnsNameServer {
    holder: Arc<FakeDnsHolder>,
}

impl FakeDnsNameServer {
    pub fn new(holder: Arc<FakeDnsHolder>) -> Self {
        Self { holder }
    }
}

#[async_trait]
impl NameServer for FakeDnsNameServer {
    fn name(&self) -> &str {
        "fakedns"
    }

    async fn query_ip(&self, domain: &str) -> Result<Vec<IpAddr>> {
        let ip = self.holder.get_fake_ip_for_domain(domain);
        Ok(vec![IpAddr::V4(ip)])
    }
}
