#[cfg(test)]
mod tests {
    use crate::app::dispatcher::DispatcherBuilder;
    use crate::app::dns::{QueryStrategy, is_local_tld_or_dotless};
    use crate::app::router::Router;
    use crate::app::router::command::CommandRoutingContext;
    use crate::common::net::Network;
    use std::net::Ipv4Addr;
    use std::sync::Arc;

    #[test]
    fn test_dns_config_and_local_tld_rules() {
        assert_eq!(QueryStrategy::from_str("use_ip4"), QueryStrategy::UseIPv4);
        assert_eq!(QueryStrategy::from_str("use_ipv6"), QueryStrategy::UseIPv6);
        assert_eq!(QueryStrategy::from_str("other"), QueryStrategy::UseIP);

        assert!(is_local_tld_or_dotless("myrouter"));
        assert!(is_local_tld_or_dotless("gateway.local"));
        assert!(is_local_tld_or_dotless("node.lan"));
        assert!(is_local_tld_or_dotless("service.home.arpa"));
        assert!(is_local_tld_or_dotless("test.example"));
        assert!(!is_local_tld_or_dotless("google.com"));
        assert!(!is_local_tld_or_dotless("github.com"));
    }

    #[test]
    fn test_command_routing_context_conversion() {
        let ctx = CommandRoutingContext {
            inbound_tag: "in-socks".into(),
            source_ip: Some(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
            source_port: 54321,
            target_ip: None,
            target_domain: Some("www.google.com".into()),
            target_port: 443,
            network: Network::Tcp,
            user: Some("test-user".into()),
        };

        let session = ctx.to_session_context();
        assert_eq!(session.inbound_tag, "in-socks");
        assert_eq!(session.destination.port, 443);
        assert_eq!(
            session.user.as_ref().map(|u| u.email.as_str()),
            Some("test-user")
        );
    }

    #[test]
    fn test_dispatcher_builder() {
        let router = Arc::new(Router::new(Vec::new(), None));
        let builder = DispatcherBuilder::new().with_router(router);
        let dispatcher = builder.build();
        assert!(dispatcher.is_some());
    }
}
