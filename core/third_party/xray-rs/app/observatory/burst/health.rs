// Module: app\observatory\burst\health.rs
// 1:1 Rust implementation corresponding to Go app\observatory\burst\health.go

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthStatus {
    pub tag: String,
    pub alive: bool,
    pub latency_ms: u64,
}

impl HealthStatus {
    pub fn new(tag: impl Into<String>, alive: bool, latency_ms: u64) -> Self {
        Self {
            tag: tag.into(),
            alive,
            latency_ms,
        }
    }
}
