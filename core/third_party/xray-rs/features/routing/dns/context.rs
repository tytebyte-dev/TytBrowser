// Module: features\routing\dns\context.rs
// 1:1 Rust implementation corresponding to Go features\routing\dns\context.go

use crate::common::net::Network;
use crate::features::dns::client::DnsClient;
use crate::features::routing::context::RoutingContext;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;

pub struct ResolvableContext {
    inner: Box<dyn RoutingContext>,
    dns_client: Arc<dyn DnsClient>,
    cached_ips: std::sync::Mutex<Option<Vec<IpAddr>>>,
}

impl ResolvableContext {
    pub fn new(inner: Box<dyn RoutingContext>, dns_client: Arc<dyn DnsClient>) -> Self {
        Self {
            inner,
            dns_client,
            cached_ips: std::sync::Mutex::new(None),
        }
    }

    pub fn dns_client(&self) -> &Arc<dyn DnsClient> {
        &self.dns_client
    }
}

impl RoutingContext for ResolvableContext {
    fn get_inbound_tag(&self) -> &str {
        self.inner.get_inbound_tag()
    }

    fn get_source_ips(&self) -> Vec<IpAddr> {
        self.inner.get_source_ips()
    }

    fn get_source_port(&self) -> u16 {
        self.inner.get_source_port()
    }

    fn get_target_ips(&self) -> Vec<IpAddr> {
        {
            let guard = self.cached_ips.lock().unwrap();
            if let Some(ref ips) = *guard {
                return ips.clone();
            }
        }

        let existing = self.inner.get_target_ips();
        if !existing.is_empty() {
            let mut guard = self.cached_ips.lock().unwrap();
            *guard = Some(existing.clone());
            return existing;
        }

        Vec::new()
    }

    fn get_target_port(&self) -> u16 {
        self.inner.get_target_port()
    }

    fn get_target_domain(&self) -> Option<&str> {
        self.inner.get_target_domain()
    }

    fn get_network(&self) -> Network {
        self.inner.get_network()
    }

    fn get_protocol(&self) -> Option<&str> {
        self.inner.get_protocol()
    }

    fn get_user(&self) -> Option<&str> {
        self.inner.get_user()
    }

    fn get_attributes(&self) -> &HashMap<String, String> {
        self.inner.get_attributes()
    }

    fn get_skip_dns_resolve(&self) -> bool {
        self.inner.get_skip_dns_resolve()
    }
}
