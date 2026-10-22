// Module: transport\internet\hysteria\congestion\utils.rs
// 1:1 Rust implementation corresponding to Go transport\internet\hysteria\congestion\utils.go

use std::time::Duration;

pub fn calculate_bandwidth(bytes: u64, elapsed: Duration) -> u64 {
    let secs = elapsed.as_secs_f64();
    if secs > 0.0 {
        ((bytes as f64) / secs) as u64
    } else {
        0
    }
}
