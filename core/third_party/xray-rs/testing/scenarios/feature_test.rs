// Module: testing\scenarios\feature_test.rs
#[cfg(test)]
mod tests {
    use crate::features::stats::StatsManager;

    #[test]
    fn test_scenario_stats_counter() {
        let mgr = StatsManager::new();
        let counter = mgr.register_counter("test_metric");
        assert_eq!(counter.add(10), 10);
        assert_eq!(counter.value(), 10);
    }
}
