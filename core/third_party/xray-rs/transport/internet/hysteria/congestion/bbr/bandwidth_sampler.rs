// Module: transport\internet\hysteria\congestion\bbr\bandwidth_sampler.rs
// 1:1 Rust implementation corresponding to Go transport\internet\hysteria\congestion\bbr\bandwidth_sampler.go

use std::time::Duration;

#[derive(Debug, Clone, Default)]
pub struct BandwidthSample {
    pub bandwidth: u64,
    pub rtt: Duration,
}

pub struct BandwidthSampler {
    samples: Vec<BandwidthSample>,
}

impl BandwidthSampler {
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
        }
    }

    pub fn record(&mut self, bandwidth: u64, rtt: Duration) {
        self.samples.push(BandwidthSample { bandwidth, rtt });
    }

    pub fn max_bandwidth(&self) -> u64 {
        self.samples.iter().map(|s| s.bandwidth).max().unwrap_or(0)
    }
}

impl Default for BandwidthSampler {
    fn default() -> Self {
        Self::new()
    }
}
