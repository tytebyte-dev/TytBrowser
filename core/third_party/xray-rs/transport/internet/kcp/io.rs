// Module: transport\internet\kcp\io.rs
// 1:1 Rust implementation corresponding to Go transport\internet\kcp\io.go

use std::collections::VecDeque;

pub struct KcpIoQueue {
    queue: VecDeque<Vec<u8>>,
}

impl KcpIoQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn push(&mut self, data: Vec<u8>) {
        self.queue.push_back(data);
    }

    pub fn pop(&mut self) -> Option<Vec<u8>> {
        self.queue.pop_front()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }
}

impl Default for KcpIoQueue {
    fn default() -> Self {
        Self::new()
    }
}
