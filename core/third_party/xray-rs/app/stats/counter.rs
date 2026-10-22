// Module: app\stats\counter.rs
// 1:1 Rust implementation corresponding to Go app\stats\counter.go

use std::sync::atomic::{AtomicI64, Ordering};

#[derive(Debug, Default)]
pub struct Counter {
    value: AtomicI64,
}

impl Counter {
    pub fn new() -> Self {
        Self {
            value: AtomicI64::new(0),
        }
    }

    pub fn value(&self) -> i64 {
        self.value.load(Ordering::Relaxed)
    }

    pub fn set(&self, new_val: i64) -> i64 {
        self.value.swap(new_val, Ordering::Relaxed)
    }

    pub fn add(&self, delta: i64) -> i64 {
        self.value.fetch_add(delta, Ordering::Relaxed) + delta
    }
}
