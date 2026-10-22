// Module: transport\internet\hysteria\congestion\bbr\windowed_filter.rs
// 1:1 Rust implementation corresponding to Go transport\internet\hysteria\congestion\bbr\windowed_filter.go

use std::time::{Duration, Instant};

pub struct WindowedMaxFilter {
    window_length: Duration,
    best_value: u64,
    best_time: Instant,
}

impl WindowedMaxFilter {
    pub fn new(window_length: Duration) -> Self {
        Self {
            window_length,
            best_value: 0,
            best_time: Instant::now(),
        }
    }

    pub fn update(&mut self, val: u64, now: Instant) {
        if val >= self.best_value || now.duration_since(self.best_time) > self.window_length {
            self.best_value = val;
            self.best_time = now;
        }
    }

    pub fn get_best(&self) -> u64 {
        self.best_value
    }
}
