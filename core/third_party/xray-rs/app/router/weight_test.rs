// Module: app\router\weight_test.rs
// 1:1 Rust unit test suite corresponding to Go app\router\weight_test.go

#[cfg(test)]
mod tests {
    use super::super::weight::{StrategyWeight, WeightManager};

    #[test]
    fn test_weight_manager_lookup() {
        let weights = vec![StrategyWeight::new("outbound-", 50.0)];
        let wm = WeightManager::new(weights, 1.0, |val, weight| val * weight);

        let w = wm.get("outbound-us");
        assert_eq!(w, 50.0);

        let default_w = wm.get("direct");
        assert_eq!(default_w, 1.0);

        let scaled = wm.apply("outbound-us", 2.0);
        assert_eq!(scaled, 100.0);
    }

    #[test]
    fn test_weight_manager_regex_and_auto_parse() {
        let weights = vec![
            StrategyWeight::regex(r"node-(\d+)", 0.0), // value 0 means auto-parse from matched number
        ];
        let wm = WeightManager::new(weights, 1.0, |val, weight| val * weight);

        let w = wm.get("node-250");
        assert_eq!(w, 250.0);

        let default_w = wm.get("unmatched");
        assert_eq!(default_w, 1.0);
    }
}
