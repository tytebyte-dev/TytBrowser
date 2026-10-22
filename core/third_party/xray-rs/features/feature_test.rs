// Module: features\feature_test.rs
// Comprehensive unit test suite for Xray features

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};
    use std::sync::Arc;
    use std::time::Duration;

    use crate::common::net::{Address, Destination, Network};
    use crate::common::protocol::{SessionContext, User};
    use crate::features::dns::{
        DEFAULT_TTL, DnsClient, FAKE_IPV4_POOL, FAKE_IPV6_POOL, FakeDnsEngine, FakeDnsFeature,
        IPOption, LocalDnsClient, RCodeError, client_type, fake_dns_type,
    };
    use crate::features::extension::{
        ContextReceiver, Observation, ObservatoryFeature, observatory_type,
    };
    use crate::features::feature::{
        Feature, TYPE_DISPATCHER, TYPE_DNS_CLIENT, TYPE_FAKE_DNS, TYPE_INBOUND_MANAGER,
        TYPE_OBSERVATORY, TYPE_OUTBOUND_MANAGER, TYPE_POLICY_MANAGER, TYPE_ROUTER,
        TYPE_STATS_MANAGER,
    };
    use crate::features::policy::{
        DefaultPolicyManager, PolicyManager, Timeout, default_buffer_policy, default_policy,
        manager_type as policy_manager_type, session_default,
    };
    use crate::features::routing::{
        DefaultRoute, DefaultRouter, ResolvableContext, Route, RouteContext, RouterFeature,
        RoutingContext, SessionRouteContext, dispatcher_type, router_type,
    };
    use crate::features::stats::{
        Counter, DefaultOnlineMap, DefaultStatsManager, NoopStatsManager, OnlineMap,
        get_or_register_counter, get_or_register_online_map, manager_type as stats_manager_type,
    };

    #[test]
    fn test_feature_type_constants() {
        assert_eq!(TYPE_DNS_CLIENT, "dns_client");
        assert_eq!(TYPE_INBOUND_MANAGER, "inbound_manager");
        assert_eq!(TYPE_OUTBOUND_MANAGER, "outbound_manager");
        assert_eq!(TYPE_POLICY_MANAGER, "policy_manager");
        assert_eq!(TYPE_ROUTER, "router");
        assert_eq!(TYPE_DISPATCHER, "dispatcher");
        assert_eq!(TYPE_STATS_MANAGER, "stats_manager");
        assert_eq!(TYPE_OBSERVATORY, "observatory");
        assert_eq!(TYPE_FAKE_DNS, "fake_dns");

        assert_eq!(client_type(), TYPE_DNS_CLIENT);
        assert_eq!(fake_dns_type(), TYPE_FAKE_DNS);
        assert_eq!(observatory_type(), TYPE_OBSERVATORY);
        assert_eq!(policy_manager_type(), TYPE_POLICY_MANAGER);
        assert_eq!(router_type(), TYPE_ROUTER);
        assert_eq!(dispatcher_type(), TYPE_DISPATCHER);
        assert_eq!(stats_manager_type(), TYPE_STATS_MANAGER);
    }

    #[tokio::test]
    async fn test_dns_localdns_and_options() {
        let client = LocalDnsClient::new();
        assert_eq!(client.feature_type(), TYPE_DNS_CLIENT);
        assert!(client.start().is_ok());
        assert!(client.close().is_ok());

        let opt = IPOption::default();
        assert!(opt.ipv4_enable);
        assert!(opt.ipv6_enable);
        assert!(!opt.fake_enable);

        let (ips, ttl) = client
            .lookup_ip_with_option("127.0.0.1", opt)
            .await
            .unwrap();
        assert_eq!(ttl, DEFAULT_TTL);
        assert!(ips.contains(&IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))));

        let rcode_err = RCodeError(3);
        assert_eq!(format!("{}", rcode_err), "rcode: 3");
    }

    struct MockFakeDns {
        domain: String,
        ip: IpAddr,
    }

    impl Feature for MockFakeDns {
        fn feature_type(&self) -> &'static str {
            TYPE_FAKE_DNS
        }
    }

    impl FakeDnsEngine for MockFakeDns {
        fn get_fake_ip_for_domain(&self, domain: &str) -> Vec<IpAddr> {
            if domain == self.domain {
                vec![self.ip]
            } else {
                Vec::new()
            }
        }

        fn get_domain_from_fake_dns(&self, ip: &IpAddr) -> Option<String> {
            if *ip == self.ip {
                Some(self.domain.clone())
            } else {
                None
            }
        }

        fn is_ip_in_ip_pool(&self, ip: &IpAddr) -> bool {
            *ip == self.ip
        }
    }

    #[test]
    fn test_fake_dns_engine_and_pools() {
        assert_eq!(FAKE_IPV4_POOL, "198.18.0.0/15");
        assert_eq!(FAKE_IPV6_POOL, "fc00::/18");

        let engine = MockFakeDns {
            domain: "google.com".into(),
            ip: IpAddr::V4(Ipv4Addr::new(198, 18, 0, 1)),
        };

        assert_eq!(
            engine.get_fake_ip_for_domain("google.com"),
            vec![IpAddr::V4(Ipv4Addr::new(198, 18, 0, 1))]
        );
        assert_eq!(
            engine.get_domain_from_fake_dns(&IpAddr::V4(Ipv4Addr::new(198, 18, 0, 1))),
            Some("google.com".into())
        );
        assert!(engine.is_ip_in_ip_pool(&IpAddr::V4(Ipv4Addr::new(198, 18, 0, 1))));

        // Test FakeDnsFeature adapter methods
        assert_eq!(
            engine.query_ip("google.com"),
            Some(IpAddr::V4(Ipv4Addr::new(198, 18, 0, 1)))
        );
        assert_eq!(
            engine.query_domain(&IpAddr::V4(Ipv4Addr::new(198, 18, 0, 1))),
            Some("google.com".into())
        );
    }

    struct MockObservatory;
    impl Feature for MockObservatory {
        fn feature_type(&self) -> &'static str {
            TYPE_OBSERVATORY
        }
    }
    #[async_trait::async_trait]
    impl ObservatoryFeature for MockObservatory {
        async fn get_observation(
            &self,
            tag: &str,
        ) -> crate::common::errors::Result<Option<Observation>> {
            Ok(Some(Observation {
                tag: tag.to_string(),
                latency: Some(Duration::from_millis(50)),
                last_seen: 12345678,
            }))
        }
        async fn list_observations(&self) -> Vec<Observation> {
            Vec::new()
        }
    }

    struct MockReceiver;
    impl ContextReceiver for MockReceiver {
        fn receive_context(&self, _ctx: &SessionContext) {}
    }

    #[tokio::test]
    async fn test_extension_features() {
        let obs = MockObservatory;
        let res = obs.get_observation("out-1").await.unwrap().unwrap();
        assert_eq!(res.tag, "out-1");
        assert_eq!(res.latency, Some(Duration::from_millis(50)));
        assert_eq!(res.last_seen, 12345678);

        let tags = vec!["proxy".to_string(), "direct".to_string()];
        assert_eq!(obs.select_outbound(&tags), Some("proxy".to_string()));

        let receiver = MockReceiver;
        let sess = SessionContext::new(
            "in-1",
            Destination {
                network: Network::Tcp,
                address: Address::ip(IpAddr::V4(Ipv4Addr::LOCALHOST)),
                port: 80,
            },
        );
        receiver.receive_context(&sess);
        receiver.inject_context("test-tag");
    }

    #[test]
    fn test_policy_defaults_and_manager() {
        let def_timeout = Timeout::default();
        assert_eq!(def_timeout.handshake, Duration::from_secs(60));
        assert_eq!(def_timeout.connection_idle, Duration::from_secs(300));
        assert_eq!(def_timeout.uplink_only, Duration::from_secs(1));
        assert_eq!(def_timeout.downlink_only, Duration::from_secs(1));

        let def_buf = default_buffer_policy();
        assert_eq!(def_buf.per_connection, 512 * 1024);

        let sess_def = session_default();
        assert_eq!(sess_def.timeouts.connection_idle, Duration::from_secs(300));

        let mgr = DefaultPolicyManager::new();
        assert_eq!(mgr.feature_type(), TYPE_POLICY_MANAGER);

        let p0 = mgr.for_level(0);
        assert_eq!(p0.timeouts.connection_idle, Duration::from_secs(300));

        let p1 = mgr.for_level(1);
        assert_eq!(p1.timeouts.connection_idle, Duration::from_secs(600));

        let sys = mgr.for_system();
        assert!(!sys.stats.inbound_uplink);

        let p = default_policy();
        assert_eq!(p.timeouts.handshake, Duration::from_secs(60));
    }

    #[test]
    fn test_routing_contexts_and_router() {
        let dest = Destination {
            network: Network::Tcp,
            address: Address::domain("example.com"),
            port: 443,
        };

        let mut route_ctx = RouteContext::new(dest.clone());
        route_ctx.inbound_tag = "socks-in".into();
        route_ctx.source = Some("127.0.0.1:54321".parse().unwrap());
        route_ctx.protocol = Some("http".into());
        route_ctx.user_email = Some("user@test.com".into());

        assert_eq!(route_ctx.get_inbound_tag(), "socks-in");
        assert_eq!(route_ctx.get_source_port(), 54321);
        assert_eq!(route_ctx.get_target_port(), 443);
        assert_eq!(route_ctx.get_target_domain(), Some("example.com"));
        assert_eq!(route_ctx.get_protocol(), Some("http"));
        assert_eq!(route_ctx.get_user(), Some("user@test.com"));
        assert_eq!(route_ctx.get_network(), Network::Tcp);
        assert!(!route_ctx.get_skip_dns_resolve());

        let mut sess = SessionContext::new("dokodemo-in", dest);
        sess.source = Some("10.0.0.1:12345".parse().unwrap());
        sess.user = Some(User {
            id: uuid::Uuid::nil(),
            email: "sess_user@test.com".into(),
            level: 1,
        });

        let sess_route_ctx = SessionRouteContext::new(sess);
        assert_eq!(sess_route_ctx.get_inbound_tag(), "dokodemo-in");
        assert_eq!(sess_route_ctx.get_source_port(), 12345);
        assert_eq!(sess_route_ctx.get_user(), Some("sess_user@test.com"));

        let resolvable =
            ResolvableContext::new(Box::new(route_ctx), Arc::new(LocalDnsClient::new()));
        assert_eq!(resolvable.get_inbound_tag(), "socks-in");
        assert_eq!(resolvable.get_target_domain(), Some("example.com"));

        let default_router = DefaultRouter;
        assert_eq!(default_router.feature_type(), TYPE_ROUTER);
        assert!(
            default_router
                .pick_outbound(&SessionContext::new(
                    "in",
                    Destination {
                        network: Network::Tcp,
                        address: Address::ip(IpAddr::V4(Ipv4Addr::LOCALHOST)),
                        port: 80,
                    }
                ))
                .is_none()
        );

        let def_route = DefaultRoute {
            tag: "out-proxy".into(),
            rule: Some("rule-1".into()),
            group_tags: vec!["group-a".into()],
        };
        assert_eq!(def_route.outbound_tag(), "out-proxy");
        assert_eq!(def_route.rule_tag(), Some("rule-1"));
        assert_eq!(def_route.outbound_group_tags(), &["group-a".to_string()]);
    }

    #[test]
    fn test_stats_counter_online_map_and_manager() {
        let counter = Counter::new();
        assert_eq!(counter.value(), 0);
        assert_eq!(counter.add(5), 5);
        assert_eq!(counter.value(), 5);
        assert_eq!(counter.set(20), 5);
        assert_eq!(counter.value(), 20);

        let online_map = DefaultOnlineMap::new();
        assert_eq!(online_map.count(), 0);
        online_map.add_ip("192.168.1.1");
        online_map.add_ip("192.168.1.1");
        assert_eq!(online_map.count(), 1);
        assert!(online_map.list().contains(&"192.168.1.1".to_string()));

        online_map.remove_ip("192.168.1.1");
        assert_eq!(online_map.count(), 1);
        online_map.remove_ip("192.168.1.1");
        assert_eq!(online_map.count(), 0);

        let mgr = DefaultStatsManager::new();
        assert_eq!(mgr.feature_type(), TYPE_STATS_MANAGER);

        let c1 = mgr.register_counter("bytes_sent");
        c1.add(1024);
        assert_eq!(mgr.get_counter("bytes_sent").unwrap().value(), 1024);

        let c1_again = get_or_register_counter(&mgr, "bytes_sent");
        assert_eq!(c1_again.value(), 1024);

        let m1 = mgr.register_online_map("user_online");
        m1.add_ip("1.2.3.4");
        assert_eq!(mgr.get_online_map("user_online").unwrap().count(), 1);

        let m1_again = get_or_register_online_map(&mgr, "user_online");
        assert_eq!(m1_again.count(), 1);

        assert!(
            mgr.get_all_online_users()
                .contains(&"user_online".to_string())
        );

        let ch = mgr.register_channel("traffic_channel");
        ch.publish(2048);
        assert_eq!(ch.subscriber_count(), 0);

        let noop = NoopStatsManager;
        assert_eq!(noop.feature_type(), TYPE_STATS_MANAGER);
        assert!(noop.get_counter("any").is_none());
        assert!(noop.get_online_map("any").is_none());
        assert!(noop.get_all_online_users().is_empty());
    }
}
