// Module: transport\internet\hysteria\congestion\bbr\packet_number_indexed_queue.rs
// 1:1 Rust implementation corresponding to Go transport\internet\hysteria\congestion\bbr\packet_number_indexed_queue.go

use std::collections::BTreeMap;

pub struct PacketNumberIndexedQueue<T> {
    entries: BTreeMap<u64, T>,
}

impl<T> PacketNumberIndexedQueue<T> {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, pn: u64, entry: T) {
        self.entries.insert(pn, entry);
    }

    pub fn remove(&mut self, pn: u64) -> Option<T> {
        self.entries.remove(&pn)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl<T> Default for PacketNumberIndexedQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}
