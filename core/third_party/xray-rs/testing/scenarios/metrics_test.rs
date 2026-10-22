// Module: testing\scenarios\metrics_test.rs
// Test Prometheus metrics collection and formatting

#[cfg(test)]
mod tests {
    use crate::app::metrics::MetricsHandler;
    use crate::app::stats::StatsManager;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_scenario_metrics_service() {
        let stats = Arc::new(StatsManager::new());
        let counter_up = stats
            .register_counter("inbound>>>socks-in>>>traffic>>>uplink")
            .await;
        counter_up.add(1048576);

        let counter_down = stats
            .register_counter("inbound>>>socks-in>>>traffic>>>downlink")
            .await;
        counter_down.add(2097152);

        let handler = MetricsHandler::new("metrics-in".into(), "127.0.0.1:9090".into(), stats);
        let prom = handler.format_prometheus_metrics().await;

        assert!(prom.contains("xray_traffic_bytes_total"));
        assert!(prom.contains("1048576"));
        assert!(prom.contains("2097152"));
    }
}
