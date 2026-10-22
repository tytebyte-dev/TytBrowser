// Module: app\\proxyman\\command\\command.rs
// 1:1 Rust implementation corresponding to Go app\\proxyman\\command\\command.go

use std::collections::HashSet;
use std::sync::RwLock;

pub struct ProxymanCommandService {
    inbounds: RwLock<HashSet<String>>,
    outbounds: RwLock<HashSet<String>>,
}

impl ProxymanCommandService {
    pub fn new() -> Self {
        Self {
            inbounds: RwLock::new(HashSet::new()),
            outbounds: RwLock::new(HashSet::new()),
        }
    }

    pub fn add_inbound(&self, tag: &str) -> bool {
        let mut guard = self.inbounds.write().unwrap();
        guard.insert(tag.to_string())
    }

    pub fn remove_inbound(&self, tag: &str) -> bool {
        let mut guard = self.inbounds.write().unwrap();
        guard.remove(tag)
    }

    pub fn add_outbound(&self, tag: &str) -> bool {
        let mut guard = self.outbounds.write().unwrap();
        guard.insert(tag.to_string())
    }

    pub fn remove_outbound(&self, tag: &str) -> bool {
        let mut guard = self.outbounds.write().unwrap();
        guard.remove(tag)
    }

    pub fn has_inbound(&self, tag: &str) -> bool {
        let guard = self.inbounds.read().unwrap();
        guard.contains(tag)
    }

    pub fn has_outbound(&self, tag: &str) -> bool {
        let guard = self.outbounds.read().unwrap();
        guard.contains(tag)
    }
}

impl Default for ProxymanCommandService {
    fn default() -> Self {
        Self::new()
    }
}
