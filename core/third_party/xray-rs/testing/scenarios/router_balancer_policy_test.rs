#[cfg(test)]
mod tests {
    use crate::app::metrics::MetricsOutbound;
    use crate::app::policy::{PolicyManager, SessionPolicy};
    use crate::app::reverse::ReverseConfig;
    use crate::app::router::balancing::{Balancer, RandomStrategy, RoundRobinStrategy};
    use crate::app::stats::Counter;
    use crate::common::errors::Result;
    use crate::common::net::{Address, BoxStream, Destination};
    use crate::common::protocol::SessionContext;
    use crate::features::outbound::OutboundHandler;
    use async_trait::async_trait;
    use std::sync::Arc;
    use std::time::Duration;

    struct DummyOutbound {
        tag: String,
    }

    #[async_trait]
    impl OutboundHandler for DummyOutbound {
        fn tag(&self) -> &str {
            &self.tag
        }
        async fn connect(&self, _session: &SessionContext) -> Result<BoxStream> {
            let (client, _) = tokio::io::duplex(64);
            Ok(Box::pin(client))
        }
        async fn start(&self) -> Result<()> {
            Ok(())
        }
        async fn close(&self) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_router_balancer_round_robin_and_random() {
        let rr = Arc::new(RoundRobinStrategy::new(Some("direct".into())));
        let balancer = Balancer::new(
            "balancer-nodes",
            vec!["proxy-".into()],
            rr,
            Some("direct".into()),
        );

        let candidates = vec![
            "proxy-1".to_string(),
            "proxy-2".to_string(),
            "direct".to_string(),
        ];

        let pick1 = balancer.pick_outbound(&candidates).unwrap();
        let pick2 = balancer.pick_outbound(&candidates).unwrap();
        let pick3 = balancer.pick_outbound(&candidates).unwrap();

        assert_eq!(pick1, "proxy-1");
        assert_eq!(pick2, "proxy-2");
        assert_eq!(pick3, "proxy-1");

        let rnd = Arc::new(RandomStrategy::new());
        let rnd_balancer = Balancer::new("balancer-rnd", vec!["proxy-".into()], rnd, None);
        let picked_rnd = rnd_balancer.pick_outbound(&candidates).unwrap();
        assert!(picked_rnd.starts_with("proxy-"));
    }

    #[test]
    fn test_policy_manager_custom_levels() {
        let mut mgr = PolicyManager::default();
        let mut p = SessionPolicy::default();
        p.buffer_size = 1024 * 1024;
        p.conn_idle = Duration::from_secs(600);
        mgr.set_level(5, p);

        let retrieved = mgr.for_level(5);
        assert_eq!(retrieved.buffer_size, 1024 * 1024);
        assert_eq!(retrieved.conn_idle, Duration::from_secs(600));

        let default_p = mgr.for_level(0);
        assert_eq!(default_p.buffer_size, 512 * 1024);
    }

    #[test]
    fn test_reverse_config_builder() {
        let mut cfg = ReverseConfig::new();
        cfg.add_bridge("bridge-1", "reverse.local");
        cfg.add_portal("portal-1", "reverse.local");

        assert_eq!(cfg.bridges.len(), 1);
        assert_eq!(cfg.portals.len(), 1);
        assert_eq!(cfg.bridges[0].tag, "bridge-1");
        assert_eq!(cfg.portals[0].tag, "portal-1");
    }

    #[tokio::test]
    async fn test_metrics_outbound_stream_wrapper() {
        let inner = Arc::new(DummyOutbound {
            tag: "dummy-out".into(),
        });
        let r_counter = Arc::new(Counter::new());
        let w_counter = Arc::new(Counter::new());
        let metrics_out = MetricsOutbound::new(inner, r_counter.clone(), w_counter.clone());

        assert_eq!(metrics_out.tag(), "dummy-out");
        let dest = Destination::tcp(Address::Domain("example.com".into()), 80);
        let session = SessionContext::new("in-test", dest);
        let stream = metrics_out.connect(&session).await;
        assert!(stream.is_ok());
    }
}
