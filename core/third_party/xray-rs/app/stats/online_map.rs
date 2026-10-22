// Module: app\stats\online_map.rs
// 1:1 Rust implementation corresponding to Go app\stats\online_map.go

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::Instant;

struct IpEntry {
    ref_count: usize,
    last_seen: Instant,
}

pub struct OnlineMap {
    entries: RwLock<HashMap<String, IpEntry>>,
}

impl Default for OnlineMap {
    fn default() -> Self {
        Self::new()
    }
}

impl OnlineMap {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
        }
    }

    pub fn add_ip(&self, ip: &str) {
        if ip == "127.0.0.1" || ip == "::1" || ip == "[::1]" {
            return;
        }

        let mut map = self.entries.write().unwrap();
        let entry = map.entry(ip.to_string()).or_insert_with(|| IpEntry {
            ref_count: 0,
            last_seen: Instant::now(),
        });
        entry.ref_count += 1;
        entry.last_seen = Instant::now();
    }

    pub fn remove_ip(&self, ip: &str) {
        let mut map = self.entries.write().unwrap();
        if let Some(entry) = map.get_mut(ip) {
            if entry.ref_count > 1 {
                entry.ref_count -= 1;
            } else {
                map.remove(ip);
            }
        }
    }

    pub fn count(&self) -> usize {
        let map = self.entries.read().unwrap();
        map.len()
    }

    pub fn list(&self) -> Vec<String> {
        self.list_ips()
    }

    pub fn list_ips(&self) -> Vec<String> {
        let map = self.entries.read().unwrap();
        map.keys().cloned().collect()
    }

    pub fn ip_time_map(&self) -> HashMap<String, Instant> {
        let map = self.entries.read().unwrap();
        map.iter().map(|(k, v)| (k.clone(), v.last_seen)).collect()
    }
}
