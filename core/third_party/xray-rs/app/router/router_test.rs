// Module: app\router\router_test.rs
// 1:1 Rust unit test suite corresponding to Go app\router\router_test.go

#[cfg(test)]
mod tests {
    use super::super::condition::{DomainMatcher, Rule};
    use super::super::router::Router;
    use crate::app::observatory::Observatory;
    use crate::app::router::balancing::{
        Balancer, BalancingStrategy, LeastPingStrategy, RandomStrategy, RoundRobinStrategy,
    };
    use crate::app::router::balancing_override::BalancingOverride;
    use crate::app::router::config::RouterConfig;
    use crate::app::router::geosite_compact::GeoSiteCompactList;
    use crate::common::net::{Address, Destination};
    use crate::common::protocol::SessionContext;
    use crate::features::routing::RouterFeature;
    use std::sync::Arc;
    use std::time::Duration;

    #[test]
    fn test_router_dispatch_rule_matching() {
        let mut rule = Rule::new("direct_out");
        rule.domain_matchers
            .push(DomainMatcher::Suffix("local-host.net".into()));

        let router = Router::new(vec![rule], Some("default_out".into()));

        let dest = Destination::tcp(Address::Domain("my.local-host.net".into()), 80);
        let session = SessionContext::new("test", dest);
        let tag = router.pick_outbound(&session);
        assert_eq!(tag, Some("direct_out"));

        let dest_ext = Destination::tcp(Address::Domain("remote.google.com".into()), 80);
        let session_ext = SessionContext::new("test", dest_ext);
        let tag_ext = router.pick_outbound(&session_ext);
        assert_eq!(tag_ext, Some("default_out"));
    }

    #[test]
    fn test_router_balancer_and_strategies() {
        let obs = Arc::new(Observatory::new(
            "http://example.com",
            Duration::from_secs(10),
            vec![],
        ));
        obs.record_status("out-1", true, 30, None);
        obs.record_status("out-2", true, 10, None);

        let lp = Arc::new(LeastPingStrategy::new(obs.clone(), Some("fallback".into())));
        assert_eq!(lp.name(), "leastPing");
        let picked = lp.pick_outbound(&["out-1".into(), "out-2".into()]);
        assert_eq!(picked, Some("out-2".into()));

        let rr = Arc::new(RoundRobinStrategy::new(Some("fallback".into())));
        assert_eq!(rr.name(), "roundRobin");
        let tags = vec!["out-1".to_string(), "out-2".to_string()];
        let pick1 = rr.pick_outbound(&tags);
        let pick2 = rr.pick_outbound(&tags);
        assert_ne!(pick1, pick2);

        let rand_strat = Arc::new(RandomStrategy::with_fallback(
            Some("fallback".into()),
            Some(obs),
        ));
        assert_eq!(rand_strat.name(), "random");
        let rand_pick = rand_strat.pick_outbound(&tags);
        assert!(rand_pick.is_some());

        let balancer = Balancer::new("b1", vec!["out-".into()], lp, Some("fallback".into()));
        assert_eq!(balancer.pick_outbound(&tags), Some("out-2".into()));
    }

    #[test]
    fn test_geosite_compact_list() {
        let mut compact = GeoSiteCompactList::new();
        compact.add_site("CN", vec!["baidu.com".into(), "qq.com".into()]);
        compact.add_site("GEOSITE:CN", vec!["taobao.com".into()]);
        compact.add_dep("CN", "GEOSITE:CN");

        let domains = compact.get_all_domains("CN");
        assert_eq!(domains.len(), 3);
        assert!(domains.contains(&"baidu.com".to_string()));
        assert!(domains.contains(&"taobao.com".to_string()));
    }

    #[test]
    fn test_balancing_override_and_router_config() {
        let ov = BalancingOverride::new("balancer-1", "out-direct");
        assert_eq!(ov.tag, "balancer-1");
        assert_eq!(ov.target, "out-direct");

        let mut rc = RouterConfig::new();
        rc.domain_strategy = "AsIs".to_string();
        rc.domain_matcher = "mph".to_string();
        assert_eq!(rc.domain_strategy, "AsIs");
        assert_eq!(rc.domain_matcher, "mph");
    }
}
