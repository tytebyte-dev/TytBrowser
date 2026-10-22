// Module: testing\mocks\mux.rs
// Mock mux session

use std::sync::atomic::{AtomicU16, Ordering};

pub struct MockMuxSession {
    next_id: AtomicU16,
}

impl MockMuxSession {
    pub fn new() -> Self {
        Self {
            next_id: AtomicU16::new(1),
        }
    }

    pub fn allocate_id(&self) -> u16 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }
}

impl Default for MockMuxSession {
    fn default() -> Self {
        Self::new()
    }
}
