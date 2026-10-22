// Module: features\routing\session\context.rs
// 1:1 Rust implementation corresponding to Go features\routing\session\context.go

use crate::common::net::Network;
use crate::common::protocol::SessionContext;
use crate::features::routing::context::RoutingContext;
use std::collections::HashMap;
use std::net::IpAddr;

pub struct SessionRouteContext {
    pub session: SessionContext,
    pub attributes: HashMap<String, String>,
}

impl SessionRouteContext {
    pub fn new(session: SessionContext) -> Self {
        Self {
            session,
            attributes: HashMap::new(),
        }
    }
}

impl RoutingContext for SessionRouteContext {
    fn get_inbound_tag(&self) -> &str {
        &self.session.inbound_tag
    }

    fn get_source_ips(&self) -> Vec<IpAddr> {
        self.session
            .source
            .map(|s| vec![s.ip()])
            .unwrap_or_default()
    }

    fn get_source_port(&self) -> u16 {
        self.session.source.map(|s| s.port()).unwrap_or(0)
    }

    fn get_target_ips(&self) -> Vec<IpAddr> {
        if let Some(target) = &self.session.route_target
            && let Some(ip) = target.address.to_ip()
        {
            return vec![ip];
        }
        self.session
            .destination
            .address
            .to_ip()
            .map(|ip| vec![ip])
            .unwrap_or_default()
    }

    fn get_target_port(&self) -> u16 {
        if let Some(target) = &self.session.route_target {
            target.port
        } else {
            self.session.destination.port
        }
    }

    fn get_target_domain(&self) -> Option<&str> {
        if let Some(d) = self.session.sniffed_domain.as_deref() {
            return Some(d);
        }
        if let Some(target) = &self.session.route_target
            && let Some(d) = target.address.domain_name()
        {
            return Some(d);
        }
        self.session.destination.address.domain_name()
    }

    fn get_network(&self) -> Network {
        self.session.destination.network
    }

    fn get_protocol(&self) -> Option<&str> {
        self.session.sniffed_protocol.as_deref()
    }

    fn get_user(&self) -> Option<&str> {
        self.session.user.as_ref().map(|u| u.email.as_str())
    }

    fn get_attributes(&self) -> &HashMap<String, String> {
        &self.attributes
    }

    fn get_skip_dns_resolve(&self) -> bool {
        false
    }
}
