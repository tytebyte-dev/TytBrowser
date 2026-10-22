pub mod burst;
pub mod burstobserver;
pub mod config;
#[path = "config.pb.rs"]
pub mod config_pb;
pub mod health;
pub mod healthping;
pub mod healthping_result;
pub mod observer;
pub mod ping;
pub mod pinger;
pub mod selector;

#[cfg(test)]
pub mod burst_test;
#[cfg(test)]
pub mod healthping_result_test;

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct HealthPingSettings {
    pub destination: String,
    pub interval: Duration,
    pub sampling_count: usize,
    pub timeout: Duration,
    pub http_method: String,
}

impl Default for HealthPingSettings {
    fn default() -> Self {
        Self {
            destination: "http://www.google.com/gen_204".into(),
            interval: Duration::from_secs(30),
            sampling_count: 5,
            timeout: Duration::from_secs(5),
            http_method: "HEAD".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HealthPingResult {
    pub tag: String,
    pub samples: Vec<Duration>,
    pub last_seen: Instant,
    pub failed: bool,
}

impl HealthPingResult {
    pub fn new(tag: String) -> Self {
        Self {
            tag,
            samples: Vec::new(),
            last_seen: Instant::now(),
            failed: false,
        }
    }

    pub fn record_sample(&mut self, rtt: Duration, max_samples: usize) {
        self.samples.push(rtt);
        if self.samples.len() > max_samples {
            self.samples.remove(0);
        }
        self.last_seen = Instant::now();
        self.failed = false;
    }

    pub fn record_failure(&mut self) {
        self.failed = true;
        self.last_seen = Instant::now();
    }

    pub fn average_rtt(&self) -> Option<Duration> {
        if self.failed || self.samples.is_empty() {
            return None;
        }

        let sum: Duration = self.samples.iter().sum();
        Some(sum / (self.samples.len() as u32))
    }
}

pub struct BurstObserver {
    pub settings: HealthPingSettings,
    results: RwLock<HashMap<String, HealthPingResult>>,
}

impl BurstObserver {
    pub fn new(settings: HealthPingSettings) -> Self {
        Self {
            settings,
            results: RwLock::new(HashMap::new()),
        }
    }

    pub fn record_rtt(&self, tag: &str, rtt: Duration) {
        if let Ok(mut guard) = self.results.write() {
            let entry = guard
                .entry(tag.to_string())
                .or_insert_with(|| HealthPingResult::new(tag.to_string()));
            entry.record_sample(rtt, self.settings.sampling_count);
        }
    }

    pub fn record_failure(&self, tag: &str) {
        if let Ok(mut guard) = self.results.write() {
            let entry = guard
                .entry(tag.to_string())
                .or_insert_with(|| HealthPingResult::new(tag.to_string()));
            entry.record_failure();
        }
    }

    pub fn get_best_outbound(&self, candidates: &[String]) -> Option<String> {
        let guard = self.results.read().ok()?;
        let mut best_tag = None;
        let mut best_rtt = Duration::MAX;

        for tag in candidates {
            if let Some(res) = guard.get(tag)
                && let Some(avg) = res.average_rtt()
                && avg < best_rtt
            {
                best_rtt = avg;
                best_tag = Some(tag.clone());
            }
        }

        best_tag.or_else(|| candidates.first().cloned())
    }
}
