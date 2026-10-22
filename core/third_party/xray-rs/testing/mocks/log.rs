// Module: testing\mocks\log.rs
// Mock log message receiver

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct MockLogCollector {
    count: Arc<AtomicUsize>,
}

impl MockLogCollector {
    pub fn new() -> Self {
        Self {
            count: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn log(&self, _msg: &str) {
        self.count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn total_logs(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }
}

impl Default for MockLogCollector {
    fn default() -> Self {
        Self::new()
    }
}
