// Module: transport\internet\hysteria\congestion\bbr\bandwidth.rs
// 1:1 Rust implementation corresponding to Go transport\internet\hysteria\congestion\bbr\bandwidth.go

use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Bandwidth(pub u64); // bits per second

impl Bandwidth {
    pub const ZERO: Self = Self(0);

    pub fn from_bytes_and_duration(bytes: u64, duration: Duration) -> Self {
        if duration.is_zero() {
            return Self::ZERO;
        }
        let secs = duration.as_secs_f64();
        let bps = (bytes as f64 * 8.0) / secs;
        Self(bps as u64)
    }

    pub fn bps(&self) -> u64 {
        self.0
    }

    pub fn bytes_per_sec(&self) -> u64 {
        self.0 / 8
    }
}
