// Module: app\\stats\\command\\command.rs
// 1:1 Rust implementation corresponding to Go app\\stats\\command\\command.go

use crate::app::stats::StatsManager;
use std::collections::HashMap;
use std::sync::Arc;

pub struct StatsCommandServer {
    stats_manager: Arc<StatsManager>,
}

impl StatsCommandServer {
    pub fn new(stats_manager: Arc<StatsManager>) -> Self {
        Self { stats_manager }
    }

    pub fn get_stat_value(&self, name: &str) -> i64 {
        self.stats_manager
            .get_counter(name)
            .map(|c| c.value())
            .unwrap_or(0)
    }

    pub fn query_stats(&self, pattern: &str) -> HashMap<String, i64> {
        let counters = self.stats_manager.all_counters();
        counters
            .into_iter()
            .filter(|(name, _)| name.contains(pattern))
            .map(|(name, c)| (name, c.value()))
            .collect()
    }
}
