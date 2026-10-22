// Module: features\routing\context.rs
// 1:1 Rust implementation corresponding to Go features\routing\context.go

use crate::common::net::{Destination, Network};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};

pub trait RoutingContext: Send + Sync {
    fn get_inbound_tag(&self) -> &str;
    fn get_source_ips(&self) -> Vec<IpAddr>;
    fn get_source_port(&self) -> u16;
    fn get_target_ips(&self) -> Vec<IpAddr>;
    fn get_target_port(&self) -> u16;
    fn get_local_ips(&self) -> Vec<IpAddr> {
        Vec::new()
    }
    fn get_local_port(&self) -> u16 {
        0
    }
    fn get_target_domain(&self) -> Option<&str>;
    fn get_network(&self) -> Network;
    fn get_protocol(&self) -> Option<&str>;
    fn get_user(&self) -> Option<&str>;
    fn get_vless_route(&self) -> u16 {
        0
    }
    fn get_attributes(&self) -> &HashMap<String, String>;
    fn get_skip_dns_resolve(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone)]
pub struct RouteContext {
    pub source: Option<SocketAddr>,
    pub destination: Destination,
    pub inbound_tag: String,
    pub user_email: Option<String>,
    pub protocol: Option<String>,
    pub attributes: HashMap<String, String>,
    pub skip_dns_resolve: bool,
}

impl RouteContext {
    pub fn new(destination: Destination) -> Self {
        Self {
            source: None,
            destination,
            inbound_tag: String::new(),
            user_email: None,
            protocol: None,
            attributes: HashMap::new(),
            skip_dns_resolve: false,
        }
    }
}

impl RoutingContext for RouteContext {
    fn get_inbound_tag(&self) -> &str {
        &self.inbound_tag
    }

    fn get_source_ips(&self) -> Vec<IpAddr> {
        self.source.map(|s| vec![s.ip()]).unwrap_or_default()
    }

    fn get_source_port(&self) -> u16 {
        self.source.map(|s| s.port()).unwrap_or(0)
    }

    fn get_target_ips(&self) -> Vec<IpAddr> {
        self.destination
            .address
            .to_ip()
            .map(|ip| vec![ip])
            .unwrap_or_default()
    }

    fn get_target_port(&self) -> u16 {
        self.destination.port
    }

    fn get_target_domain(&self) -> Option<&str> {
        self.destination.address.domain_name()
    }

    fn get_network(&self) -> Network {
        self.destination.network
    }

    fn get_protocol(&self) -> Option<&str> {
        self.protocol.as_deref()
    }

    fn get_user(&self) -> Option<&str> {
        self.user_email.as_deref()
    }

    fn get_attributes(&self) -> &HashMap<String, String> {
        &self.attributes
    }

    fn get_skip_dns_resolve(&self) -> bool {
        self.skip_dns_resolve
    }
}
