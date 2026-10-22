#[cfg(test)]
mod tests {
    use crate::app::metrics::MetricsHandler;
    use crate::app::stats::StatsManager;
    use crate::app::version::{compare_versions, validate_version};
    use std::sync::Arc;

    #[test]
    fn test_version_compare_and_validate() {
        assert_eq!(compare_versions("1.8.0", "1.8.0").unwrap(), 0);
        assert_eq!(compare_versions("1.8.1", "1.8.0").unwrap(), 1);
        assert_eq!(compare_versions("1.7.9", "1.8.0").unwrap(), -1);
        assert_eq!(compare_versions("1.8", "1.8.0").unwrap(), 0);

        assert!(validate_version("1.8.23", "1.8.0", "1.9.0").is_ok());
        assert!(validate_version("1.7.0", "1.8.0", "1.9.0").is_err());
        assert!(validate_version("1.9.5", "1.8.0", "1.9.0").is_err());
    }

    #[tokio::test]
    async fn test_metrics_prometheus_and_json() {
        let stats = Arc::new(StatsManager::new());
        let in_counter = stats
            .register_counter("inbound>>>socks_in>>>traffic>>>downlink")
            .await;
        in_counter.add(1024);

        let handler = MetricsHandler::new("metrics".into(), "127.0.0.1:8080".into(), stats);
        let prom = handler.format_prometheus_metrics().await;
        assert!(prom.contains(
            "xray_traffic_bytes_total{metric=\"inbound_socks_in_traffic_downlink\"} 1024"
        ));

        let json = handler.format_json_stats().await;
        assert_eq!(
            json.get("inbound>>>socks_in>>>traffic>>>downlink"),
            Some(&1024)
        );
    }
}
