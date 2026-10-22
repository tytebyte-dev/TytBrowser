#[cfg(test)]
mod tests {
    use crate::app::dispatcher::FakeDnsSniffer;
    use crate::app::dns::fakedns::FakeDnsHolder;
    use crate::app::observatory::Observatory;
    use crate::app::router::balancing::{BalancingStrategy, LeastLoadStrategy, LeastPingStrategy};
    use std::net::{IpAddr, Ipv4Addr};
    use std::sync::Arc;
    use std::time::Duration;

    #[test]
    fn test_fakednssniffer_domain_lookup_and_pool_check() {
        let holder = Arc::new(FakeDnsHolder::new("240.0.0.0/24").unwrap());

        let sniffer = FakeDnsSniffer::new(holder.clone());
        let ip = holder.get_fake_ip_for_domain("api.google.com");
        let ip_addr = IpAddr::V4(ip);

        assert!(sniffer.is_in_pool(&ip_addr));
        assert_eq!(sniffer.sniff_domain(&ip_addr).unwrap(), "api.google.com");

        let real_ip = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
        assert!(!sniffer.is_in_pool(&real_ip));
        assert!(sniffer.sniff_domain(&real_ip).is_none());
    }

    #[test]
    fn test_least_load_and_least_ping_strategies() {
        let obs = Arc::new(Observatory::new(
            "http://cp.cloudflare.com",
            Duration::from_secs(10),
            vec!["proxy-".into()],
        ));

        obs.record_result("proxy-us", Duration::from_millis(150), true, None);
        obs.record_result("proxy-hk", Duration::from_millis(45), true, None);

        let ping_strat = LeastPingStrategy::new(obs.clone(), Some("direct".into()));
        assert_eq!(ping_strat.name(), "leastPing");
        let candidates = vec!["proxy-us".to_string(), "proxy-hk".to_string()];
        let best_ping = ping_strat.pick_outbound(&candidates).unwrap();
        assert_eq!(best_ping, "proxy-hk");

        let load_strat = LeastLoadStrategy::new(obs.clone(), Some("direct".into()));
        assert_eq!(load_strat.name(), "leastLoad");
        let best_load = load_strat.pick_outbound(&candidates).unwrap();
        assert_eq!(best_load, "proxy-hk");
    }
}
