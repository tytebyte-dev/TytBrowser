// Module: testing\mocks\dns.rs
// Mock DNS server for integration test scenarios

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::RwLock;

pub struct MockDnsServer {
    records: RwLock<HashMap<String, Vec<IpAddr>>>,
}

impl MockDnsServer {
    pub fn new() -> Self {
        Self {
            records: RwLock::new(HashMap::new()),
        }
    }

    pub fn set_record(&self, domain: &str, ips: Vec<IpAddr>) {
        let mut guard = self.records.write().unwrap();
        guard.insert(domain.to_string(), ips);
    }

    pub fn resolve(&self, domain: &str) -> Option<Vec<IpAddr>> {
        let guard = self.records.read().unwrap();
        guard.get(domain).cloned()
    }
}

impl Default for MockDnsServer {
    fn default() -> Self {
        Self::new()
    }
}
