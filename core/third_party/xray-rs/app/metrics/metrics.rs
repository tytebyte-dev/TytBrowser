// Module: app\\metrics\\metrics.rs
// 1:1 Rust implementation corresponding to Go app\\metrics\\metrics.go

pub use super::MetricsService;

pub struct MetricsCollector {
    pub service: MetricsService,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            service: MetricsService::new(),
        }
    }

    pub fn collect(&self) -> String {
        self.service.render_prometheus()
    }
}
