// Module: app\router\strategy_leastload_test.rs
// 1:1 Rust unit test suite corresponding to Go app\router\strategy_leastload_test.go

#[cfg(test)]
mod tests {
    use super::super::balancing::BalancingStrategy;
    use super::super::strategy_leastload::{
        LeastLoadNode, LeastLoadStrategy, StrategyLeastLoadConfig, leastload_sort,
        select_least_load,
    };
    use crate::app::observatory::Observatory;
    use std::sync::Arc;
    use std::time::Duration;

    #[test]
    fn test_least_load_selection() {
        let obs = Arc::new(Observatory::new(
            "http://example.com",
            Duration::from_secs(10),
            vec![],
        ));
        obs.record_status("node-1", true, 100, None);
        obs.record_status("node-2", true, 20, None);
        let strat = LeastLoadStrategy::new(obs, Some("fallback-node".into()));
        assert_eq!(strat.name(), "leastLoad");
        let chosen = strat.pick_outbound(&["node-1".into(), "node-2".into()]);
        assert_eq!(chosen, Some("node-2".into()));
    }

    #[test]
    fn test_least_load_sorting_and_baselines() {
        let mut nodes = vec![
            LeastLoadNode::new("slow", 500, 500),
            LeastLoadNode::new("fast", 50, 50),
            LeastLoadNode::new("medium", 200, 200),
        ];

        leastload_sort(&mut nodes);
        assert_eq!(nodes[0].tag, "fast");
        assert_eq!(nodes[1].tag, "medium");
        assert_eq!(nodes[2].tag, "slow");

        let baselines = vec![100, 300];
        let selected = select_least_load(&nodes, &baselines, 2);
        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0].tag, "fast");
        assert_eq!(selected[1].tag, "medium");
    }

    #[test]
    fn test_least_load_fallback_when_empty() {
        let strat = LeastLoadStrategy::with_config(
            StrategyLeastLoadConfig::default(),
            Some("my-fallback".into()),
            None,
        );
        let chosen = strat.pick_outbound(&[]);
        assert_eq!(chosen, Some("my-fallback".into()));
    }
}
