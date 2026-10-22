pub mod command;

#[cfg(test)]
pub mod command_test;

use crate::app::commander::Service;
use crate::app::stats::StatsManager;
use std::collections::HashMap;
use std::sync::Arc;

pub struct StatsService {
    stats_manager: Arc<StatsManager>,
}

impl StatsService {
    pub fn new(stats_manager: Arc<StatsManager>) -> Self {
        Self { stats_manager }
    }

    pub fn get_stat(&self, name: &str, reset: bool) -> i64 {
        if let Some(counter) = self.stats_manager.get_counter(name) {
            if reset {
                counter.set(0)
            } else {
                counter.value()
            }
        } else {
            0
        }
    }

    pub fn query_stats(&self, pattern: &str, reset: bool) -> HashMap<String, i64> {
        let mut res = HashMap::new();
        for (name, counter) in self.stats_manager.all_counters() {
            if name.contains(pattern) {
                let val = if reset {
                    counter.set(0)
                } else {
                    counter.value()
                };
                res.insert(name, val);
            }
        }
        res
    }
}

impl Service for StatsService {
    fn service_name(&self) -> &str {
        "xray.core.app.stats.command.StatsService"
    }
}
