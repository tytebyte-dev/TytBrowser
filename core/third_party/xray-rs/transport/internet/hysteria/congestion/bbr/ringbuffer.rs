// Module: transport\internet\hysteria\congestion\bbr\ringbuffer.rs
// 1:1 Rust implementation corresponding to Go transport\internet\hysteria\congestion\bbr\ringbuffer.go

use std::collections::VecDeque;

pub struct RingBuffer<T> {
    capacity: usize,
    buffer: VecDeque<T>,
}

impl<T> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            buffer: VecDeque::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, item: T) {
        if self.buffer.len() >= self.capacity {
            self.buffer.pop_front();
        }
        self.buffer.push_back(item);
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}
