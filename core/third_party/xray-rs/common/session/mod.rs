pub mod context;
pub mod session;

#[cfg(test)]
pub mod session_test;

use crate::common::net::Destination;
use crate::common::protocol::MemoryUser;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};

static SESSION_ID_COUNTER: AtomicU32 = AtomicU32::new(1000);

pub fn new_session_id() -> u32 {
    SESSION_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[derive(Debug, Clone, Default)]
pub struct Inbound {
    pub source: Option<Destination>,
    pub local: Option<Destination>,
    pub tag: String,
    pub name: String,
    pub user: Option<MemoryUser>,
    pub can_splice_copy: i32,
}

#[derive(Debug, Clone, Default)]
pub struct Outbound {
    pub original_target: Option<Destination>,
    pub target: Option<Destination>,
    pub route_target: Option<Destination>,
    pub tag: String,
    pub name: String,
    pub can_splice_copy: i32,
}

#[derive(Debug, Clone, Default)]
pub struct SniffingRequest {
    pub enabled: bool,
    pub metadata_only: bool,
    pub route_only: bool,
    pub exclude_for_domain: Vec<String>,
    pub override_destination_for_protocol: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Content {
    pub protocol: String,
    pub sniffing_request: SniffingRequest,
    pub attributes: HashMap<String, String>,
    pub skip_dns_resolve: bool,
}

impl Content {
    pub fn set_attribute(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.attributes.insert(key.into(), value.into());
    }

    pub fn attribute(&self, key: &str) -> Option<&str> {
        self.attributes.get(key).map(|s| s.as_str())
    }
}

#[derive(Debug, Clone, Default)]
pub struct Sockopt {
    pub mark: i32,
}
