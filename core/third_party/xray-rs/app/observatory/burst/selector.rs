// Module: app\observatory\burst\selector.rs
// 1:1 Rust implementation corresponding to Go app\observatory\burst\selector.go

use super::health::HealthStatus;

pub struct FastSelector;

impl FastSelector {
    pub fn select_fastest(list: &[HealthStatus]) -> Option<HealthStatus> {
        list.iter()
            .filter(|s| s.alive)
            .min_by_key(|s| s.latency_ms)
            .cloned()
    }
}

pub use FastSelector as BurstSelector;
