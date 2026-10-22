#[cfg(test)]
mod tests {
    use crate::app::dns::nameserver::{LocalNameServer, NameServer};
    use crate::app::observatory::burst::{BurstObserver, HealthPingResult, HealthPingSettings};
    use std::net::IpAddr;
    use std::time::Duration;

    #[tokio::test]
    async fn test_local_nameserver_resolution() {
        let ns = LocalNameServer::new();
        assert_eq!(ns.name(), "local");

        let ips = ns.query_ip("localhost").await;
        assert!(ips.is_ok());
        let list = ips.unwrap();
        assert!(
            list.contains(&"127.0.0.1".parse::<IpAddr>().unwrap())
                || list.contains(&"::1".parse::<IpAddr>().unwrap())
        );
    }

    #[test]
    fn test_burst_observer_rtt_and_best_selection() {
        let settings = HealthPingSettings {
            destination: "http://www.google.com/gen_204".into(),
            interval: Duration::from_secs(10),
            sampling_count: 3,
            timeout: Duration::from_secs(3),
            http_method: "HEAD".into(),
        };

        let observer = BurstObserver::new(settings);

        observer.record_rtt("proxy-fast", Duration::from_millis(50));
        observer.record_rtt("proxy-fast", Duration::from_millis(60));

        observer.record_rtt("proxy-slow", Duration::from_millis(250));
        observer.record_rtt("proxy-slow", Duration::from_millis(230));

        observer.record_failure("proxy-dead");

        let candidates = vec![
            "proxy-slow".to_string(),
            "proxy-fast".to_string(),
            "proxy-dead".to_string(),
        ];

        let best = observer.get_best_outbound(&candidates).unwrap();
        assert_eq!(best, "proxy-fast");
    }

    #[test]
    fn test_health_ping_result_average_calculation() {
        let mut res = HealthPingResult::new("node-1".into());
        res.record_sample(Duration::from_millis(100), 5);
        res.record_sample(Duration::from_millis(200), 5);
        res.record_sample(Duration::from_millis(300), 5);

        assert_eq!(res.average_rtt(), Some(Duration::from_millis(200)));

        res.record_failure();
        assert_eq!(res.average_rtt(), None);
    }
}
