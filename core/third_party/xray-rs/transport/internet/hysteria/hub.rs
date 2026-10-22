// Module: transport\internet\hysteria\hub.rs
// 1:1 Rust implementation corresponding to Go transport\internet\hysteria\hub.go

use std::collections::HashMap;
use std::sync::Mutex;

pub struct HysteriaHub {
    connections: Mutex<HashMap<u64, String>>,
}

impl HysteriaHub {
    pub fn new() -> Self {
        Self {
            connections: Mutex::new(HashMap::new()),
        }
    }

    pub fn register(&self, id: u64, tag: String) {
        let mut guard = self.connections.lock().unwrap();
        guard.insert(id, tag);
    }

    pub fn remove(&self, id: u64) {
        let mut guard = self.connections.lock().unwrap();
        guard.remove(&id);
    }
}

impl Default for HysteriaHub {
    fn default() -> Self {
        Self::new()
    }
}
