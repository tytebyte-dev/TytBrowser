// Module: common\peer\latency.rs
// 1:1 Rust implementation corresponding to Go common\peer\latency.go

use std::sync::atomic::{AtomicU64, Ordering};

pub trait Latency: Send + Sync {
    fn value(&self) -> u64;
}

pub trait HasLatency: Send + Sync {
    fn connection_latency(&self) -> &dyn Latency;
    fn handshake_latency(&self) -> &dyn Latency;
}

#[derive(Debug, Default)]
pub struct AverageLatency {
    value: AtomicU64,
}

impl AverageLatency {
    pub fn new(initial: u64) -> Self {
        Self {
            value: AtomicU64::new(initial),
        }
    }

    pub fn update(&self, new_val: u64) {
        let mut current = self.value.load(Ordering::Relaxed);
        loop {
            let next = (current + new_val * 2) / 3;
            match self.value.compare_exchange_weak(
                current,
                next,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current = actual,
            }
        }
    }

    pub fn value(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }
}

impl Latency for AverageLatency {
    fn value(&self) -> u64 {
        self.value()
    }
}
