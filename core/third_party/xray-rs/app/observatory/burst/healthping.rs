// Module: app\observatory\burst\healthping.rs
// 1:1 Rust implementation corresponding to Go app\observatory\burst\healthping.go

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::Duration;

use super::config_pb::HealthPingConfig;
use super::healthping_result::{HealthPingRTTS, HealthPingStats};

#[derive(Debug, Clone)]
pub struct HealthPingSettings {
    pub destination: String,
    pub connectivity: String,
    pub interval: Duration,
    pub sampling_count: usize,
    pub timeout: Duration,
    pub http_method: String,
}

impl Default for HealthPingSettings {
    fn default() -> Self {
        Self {
            destination: "https://connectivitycheck.gstatic.com/generate_204".into(),
            connectivity: String::new(),
            interval: Duration::from_secs(60),
            sampling_count: 10,
            timeout: Duration::from_secs(5),
            http_method: "HEAD".into(),
        }
    }
}

impl From<&HealthPingConfig> for HealthPingSettings {
    fn from(cfg: &HealthPingConfig) -> Self {
        let mut settings = Self::default();
        if !cfg.destination.is_empty() {
            settings.destination = cfg.destination.clone();
        }
        if !cfg.connectivity.is_empty() {
            settings.connectivity = cfg.connectivity.clone();
        }
        if cfg.interval > 0 {
            settings.interval = Duration::from_nanos(cfg.interval as u64);
        }
        if cfg.sampling_count > 0 {
            settings.sampling_count = cfg.sampling_count as usize;
        }
        if cfg.timeout > 0 {
            settings.timeout = Duration::from_nanos(cfg.timeout as u64);
        }
        if !cfg.http_method.is_empty() {
            settings.http_method = cfg.http_method.clone();
        }
        settings
    }
}

pub struct HealthPing {
    pub settings: HealthPingSettings,
    results: RwLock<HashMap<String, HealthPingRTTS>>,
}

impl HealthPing {
    pub fn new(settings: HealthPingSettings) -> Self {
        Self {
            settings,
            results: RwLock::new(HashMap::new()),
        }
    }

    pub fn put_rtt(&self, tag: &str, rtt: Duration) {
        let mut guard = self.results.write().unwrap();
        let entry = guard.entry(tag.to_string()).or_insert_with(|| {
            HealthPingRTTS::new(
                self.settings.sampling_count,
                self.settings.interval * (self.settings.sampling_count as u32),
            )
        });
        entry.put(rtt);
    }

    pub fn get_stats(&self, tag: &str) -> Option<HealthPingStats> {
        let mut guard = self.results.write().unwrap();
        guard.get_mut(tag).map(|entry| entry.get())
    }

    pub fn all_stats(&self) -> HashMap<String, HealthPingStats> {
        let mut guard = self.results.write().unwrap();
        let mut out = HashMap::new();
        for (tag, entry) in guard.iter_mut() {
            out.insert(tag.clone(), entry.get());
        }
        out
    }
}
