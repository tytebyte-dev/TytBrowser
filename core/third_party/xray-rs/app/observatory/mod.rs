pub mod burst;
pub mod command;
#[path = "config.pb.rs"]
pub mod config_pb;
pub mod explain_errors;
pub mod observatory;
pub mod observer;

#[cfg(test)]
pub mod observatory_test;

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub use burst::{BurstObserver, HealthPingResult, HealthPingSettings};
pub use config_pb::{ObservationResult, ProbeResult};
pub use explain_errors::{ErrorCategory, ErrorCollector, explain_error};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboundStatus {
    pub outbound_tag: String,
    pub alive: bool,
    pub delay_ms: u64,
    pub last_try_time: u64,
    pub last_seen_time: u64,
    pub last_error_reason: Option<String>,
}

impl OutboundStatus {
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            outbound_tag: tag.into(),
            alive: false,
            delay_ms: u64::MAX,
            last_try_time: 0,
            last_seen_time: 0,
            last_error_reason: None,
        }
    }
}

pub struct Observatory {
    probe_url: String,
    probe_interval: Duration,
    subject_selectors: Vec<String>,
    statuses: RwLock<HashMap<String, OutboundStatus>>,
}

impl Observatory {
    pub fn new(
        probe_url: impl Into<String>,
        probe_interval: Duration,
        subject_selectors: Vec<String>,
    ) -> Self {
        Self {
            probe_url: probe_url.into(),
            probe_interval,
            subject_selectors,
            statuses: RwLock::new(HashMap::new()),
        }
    }

    pub fn probe_url(&self) -> &str {
        &self.probe_url
    }

    pub fn probe_interval(&self) -> Duration {
        self.probe_interval
    }

    pub fn subject_selectors(&self) -> &[String] {
        &self.subject_selectors
    }

    pub fn record_result(
        &self,
        tag: &str,
        delay: Duration,
        alive: bool,
        error_reason: Option<String>,
    ) {
        self.record_status(tag, alive, delay.as_millis() as u64, error_reason);
    }

    pub fn record_status(
        &self,
        tag: &str,
        alive: bool,
        delay_ms: u64,
        error_reason: Option<String>,
    ) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if let Ok(mut guard) = self.statuses.write() {
            let entry = guard
                .entry(tag.to_string())
                .or_insert_with(|| OutboundStatus::new(tag));
            entry.alive = alive;
            entry.delay_ms = delay_ms;
            entry.last_try_time = now;
            if alive {
                entry.last_seen_time = now;
                entry.last_error_reason = None;
            } else {
                entry.last_error_reason = error_reason;
            }
        }
    }

    pub fn get_status(&self, tag: &str) -> Option<OutboundStatus> {
        let guard = self.statuses.read().ok()?;
        guard.get(tag).cloned()
    }

    pub fn get_best_outbound(&self, candidates: &[String]) -> Option<String> {
        self.select_best_outbound(candidates)
    }

    pub fn select_best_outbound(&self, candidates: &[String]) -> Option<String> {
        let guard = self.statuses.read().ok()?;
        let mut best_tag = None;
        let mut min_delay = u64::MAX;

        for tag in candidates {
            if let Some(status) = guard.get(tag)
                && status.alive
                && status.delay_ms < min_delay
            {
                min_delay = status.delay_ms;
                best_tag = Some(tag.clone());
            }
        }

        best_tag.or_else(|| candidates.first().cloned())
    }

    pub fn all_statuses(&self) -> Vec<OutboundStatus> {
        let guard = self.statuses.read().unwrap();
        guard.values().cloned().collect()
    }
}
