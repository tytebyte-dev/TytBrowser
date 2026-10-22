// Module: proxy\tun\udp_fullcone.rs
// 1:1 Rust implementation corresponding to Go proxy\tun\udp_fullcone.go

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone)]
struct NatEntry {
    target: SocketAddr,
    last_seen: Instant,
}

pub struct UdpNatTable {
    entries: Arc<Mutex<HashMap<SocketAddr, NatEntry>>>,
    ttl: Duration,
}

impl UdpNatTable {
    pub fn new(ttl: Duration) -> Self {
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
            ttl,
        }
    }

    pub fn insert(&self, src: SocketAddr, target: SocketAddr) {
        let mut guard = self.entries.lock().unwrap();
        guard.insert(
            src,
            NatEntry {
                target,
                last_seen: Instant::now(),
            },
        );
    }

    pub fn lookup(&self, src: &SocketAddr) -> Option<SocketAddr> {
        let mut guard = self.entries.lock().unwrap();
        if let Some(entry) = guard.get_mut(src)
            && entry.last_seen.elapsed() <= self.ttl
        {
            entry.last_seen = Instant::now();
            return Some(entry.target);
        }
        None
    }

    pub fn clean_expired(&self) {
        let mut guard = self.entries.lock().unwrap();
        let ttl = self.ttl;
        guard.retain(|_, v| v.last_seen.elapsed() <= ttl);
    }
}

impl Default for UdpNatTable {
    fn default() -> Self {
        Self::new(Duration::from_secs(300))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_udp_nat_table_insert_and_lookup() {
        let table = UdpNatTable::new(Duration::from_secs(10));
        let src: SocketAddr = "10.0.0.1:12345".parse().unwrap();
        let target: SocketAddr = "8.8.8.8:53".parse().unwrap();

        assert!(table.lookup(&src).is_none());
        table.insert(src, target);
        assert_eq!(table.lookup(&src), Some(target));
    }
}
