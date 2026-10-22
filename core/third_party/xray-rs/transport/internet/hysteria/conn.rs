// Module: transport\internet\hysteria\conn.rs
// 1:1 Rust implementation corresponding to Go transport\internet\hysteria\conn.go

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct HysteriaConn {
    bytes_sent: Arc<AtomicU64>,
    bytes_recv: Arc<AtomicU64>,
}

impl HysteriaConn {
    pub fn new() -> Self {
        Self {
            bytes_sent: Arc::new(AtomicU64::new(0)),
            bytes_recv: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn record_sent(&self, n: usize) {
        self.bytes_sent.fetch_add(n as u64, Ordering::Relaxed);
    }

    pub fn record_recv(&self, n: usize) {
        self.bytes_recv.fetch_add(n as u64, Ordering::Relaxed);
    }

    pub fn total_sent(&self) -> u64 {
        self.bytes_sent.load(Ordering::Relaxed)
    }

    pub fn total_recv(&self) -> u64 {
        self.bytes_recv.load(Ordering::Relaxed)
    }
}

impl Default for HysteriaConn {
    fn default() -> Self {
        Self::new()
    }
}
