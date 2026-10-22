// Module: app\observatory\burst\pinger.rs
// 1:1 Rust implementation corresponding to Go app\observatory\burst\pinger.go

use std::time::Duration;

pub struct HealthPinger {
    pub timeout: Duration,
}

impl HealthPinger {
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }

    pub fn timeout(&self) -> Duration {
        self.timeout
    }
}
