// Module: app\metrics\mod.rs
// 1:1 Rust implementation corresponding to Go app\metrics\metrics.go

#[path = "config.pb.rs"]
pub mod config;
pub mod outbound;

use crate::app::stats::StatsManager;
use std::collections::HashMap;
use std::sync::Arc;

pub use config::Config;
pub use outbound::{MetricsOutbound, Outbound, OutboundListener};

pub struct MetricsHandler {
    pub tag: String,
    pub listen: String,
    stats: Arc<StatsManager>,
}

impl MetricsHandler {
    pub fn new(tag: String, listen: String, stats: Arc<StatsManager>) -> Self {
        Self { tag, listen, stats }
    }

    pub async fn format_prometheus_metrics(&self) -> String {
        let mut out = String::new();
        out.push_str("# HELP xray_traffic_bytes_total Total bytes transferred by Xray\n");
        out.push_str("# TYPE xray_traffic_bytes_total counter\n");

        let stats_map = self.stats.get_all_stats().await;
        for (name, val) in stats_map {
            let sanitized = name.replace(">>>", "_").replace(['.', '-'], "_");
            out.push_str(&format!(
                "xray_traffic_bytes_total{{metric=\"{}\"}} {}\n",
                sanitized, val
            ));
        }

        out
    }

    pub async fn format_json_stats(&self) -> HashMap<String, i64> {
        self.stats.get_all_stats().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::net::{Address, Destination};
    use crate::common::protocol::SessionContext;
    use crate::features::outbound::OutboundHandler;

    #[test]
    fn test_metrics_config_serde() {
        let cfg = Config::new("metrics-tag", "127.0.0.1:8080");
        assert_eq!(cfg.tag, "metrics-tag");
        assert_eq!(cfg.listen, "127.0.0.1:8080");

        let json = serde_json::to_string(&cfg).expect("serialize");
        let parsed: Config = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, cfg);
    }

    #[tokio::test]
    async fn test_outbound_listener_lifecycle() {
        let listener = Arc::new(OutboundListener::new(2));
        assert!(!listener.is_closed());

        let outbound = Outbound::new("metrics-out", listener.clone());
        assert_eq!(outbound.tag(), "metrics-out");
        outbound.start().await.expect("start");

        let dest = Destination::tcp(Address::Domain("metrics.local".into()), 80);
        let session = SessionContext::new("inbound-tag", dest);
        let client_stream = outbound.connect(&session).await;
        assert!(client_stream.is_ok());

        let accepted = listener.accept().await;
        assert!(accepted.is_ok());

        outbound.close().await.expect("close");
        assert!(listener.is_closed());

        let failed_connect = outbound.connect(&session).await;
        assert!(failed_connect.is_err());
    }

    #[tokio::test]
    async fn test_metrics_handler_formatting() {
        let stats = Arc::new(StatsManager::new());
        let counter = stats
            .register_counter("inbound>>>tag>>>traffic>>>downlink")
            .await;
        counter.add(1024);

        let handler = MetricsHandler::new("metrics".into(), "127.0.0.1:9090".into(), stats);
        let prom = handler.format_prometheus_metrics().await;
        assert!(prom.contains("xray_traffic_bytes_total"));
        assert!(prom.contains("inbound_tag_traffic_downlink"));
        assert!(prom.contains("1024"));

        let json_stats = handler.format_json_stats().await;
        assert_eq!(
            json_stats.get("inbound>>>tag>>>traffic>>>downlink"),
            Some(&1024)
        );
    }
}
