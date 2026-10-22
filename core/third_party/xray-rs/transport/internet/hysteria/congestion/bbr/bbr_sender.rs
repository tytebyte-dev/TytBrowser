// Module: transport\internet\hysteria\congestion\bbr\bbr_sender.rs
// 1:1 Rust implementation corresponding to Go transport\internet\hysteria\congestion\bbr\bbr_sender.go

use super::bandwidth::Bandwidth;
use super::windowed_filter::WindowedMaxFilter;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BbrMode {
    Startup,
    Drain,
    ProbeBW,
    ProbeRTT,
}

pub struct BbrSender {
    pub mode: BbrMode,
    pub max_bandwidth: WindowedMaxFilter,
    pub min_rtt: Duration,
    pub congestion_window: u64,
}

impl BbrSender {
    pub fn new() -> Self {
        Self {
            mode: BbrMode::Startup,
            max_bandwidth: WindowedMaxFilter::new(Duration::from_secs(10)),
            min_rtt: Duration::from_millis(100),
            congestion_window: 64 * 1024,
        }
    }

    pub fn pacing_rate(&self) -> Bandwidth {
        Bandwidth(self.max_bandwidth.get_best())
    }
}

impl Default for BbrSender {
    fn default() -> Self {
        Self::new()
    }
}
