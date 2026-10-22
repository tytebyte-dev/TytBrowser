// Module: app\observatory\burst\burstobserver.rs
// 1:1 Rust implementation corresponding to Go app\observatory\burst\burstobserver.go

use super::config_pb::Config;
use super::healthping::{HealthPing, HealthPingSettings};
use crate::app::observatory::config_pb::{
    HealthPingMeasurementResult, ObservationResult, OutboundStatus,
};
use std::sync::Arc;

/// Observer implements health monitoring of outbounds using burst pings.
pub struct Observer {
    pub config: Option<Config>,
    pub hp: Arc<HealthPing>,
}

impl Observer {
    pub fn new(config: Option<Config>) -> Self {
        let settings = match &config {
            Some(c) => match &c.ping_config {
                Some(pc) => HealthPingSettings::from(pc),
                None => HealthPingSettings::default(),
            },
            None => HealthPingSettings::default(),
        };

        Self {
            config,
            hp: Arc::new(HealthPing::new(settings)),
        }
    }

    pub fn get_observation(&self) -> ObservationResult {
        ObservationResult {
            status: self.create_result(),
        }
    }

    pub fn record_rtt(&self, tag: &str, rtt: std::time::Duration) {
        self.hp.put_rtt(tag, rtt);
    }

    pub fn create_result(&self) -> Vec<OutboundStatus> {
        let all_stats = self.hp.all_stats();
        let mut results = Vec::new();

        for (name, stats) in all_stats {
            let alive = stats.all != stats.fail && stats.all > 0;
            let status = OutboundStatus {
                alive,
                delay: stats.average.as_millis() as i64,
                last_error_reason: String::new(),
                outbound_tag: name,
                last_seen_time: 0,
                last_try_time: 0,
                health_ping: Some(HealthPingMeasurementResult {
                    all: stats.all as i64,
                    fail: stats.fail as i64,
                    deviation: stats.deviation.as_nanos() as i64,
                    average: stats.average.as_nanos() as i64,
                    max: stats.max.as_nanos() as i64,
                    min: stats.min.as_nanos() as i64,
                }),
            };
            results.push(status);
        }

        results
    }
}

// Backward compatibility alias
pub type BurstObserver = Observer;
