#[cfg(test)]
mod tests {
    use crate::app::proxyman::inbound::InboundWorker;
    use crate::app::router::webhook::{WebhookPayload, WebhookSender};
    use crate::app::router::weight::{StrategyWeight, WeightManager};

    #[test]
    fn test_weight_manager_pattern_matching_and_scaling() {
        let weights = vec![
            StrategyWeight::new("proxy-us", 2.0),
            StrategyWeight::new("proxy-hk", 1.5),
        ];

        let mgr = WeightManager::new(weights, 1.0, |val, w| val * w);

        assert_eq!(mgr.get("proxy-us-01"), 2.0);
        assert_eq!(mgr.get("proxy-hk-01"), 1.5);
        assert_eq!(mgr.get("direct"), 1.0);

        assert_eq!(mgr.apply("proxy-us-01", 10.0), 20.0);
        assert_eq!(mgr.apply("direct", 10.0), 10.0);
    }

    #[test]
    fn test_webhook_sender_and_payload_serialization() {
        let sender = WebhookSender::new("https://example.com/webhook");
        assert_eq!(sender.url(), "https://example.com/webhook");

        let payload = WebhookPayload {
            inbound_tag: "in-socks".into(),
            outbound_tag: "proxy-us".into(),
            destination: "google.com:443".into(),
            source: Some("127.0.0.1:12345".into()),
        };

        let json_str = sender.format_payload(&payload).unwrap();
        assert!(json_str.contains("in-socks"));
        assert!(json_str.contains("proxy-us"));
        assert!(json_str.contains("google.com:443"));
    }

    #[test]
    fn test_inbound_worker_lifecycle() {
        let worker = InboundWorker::new("in-socks", "127.0.0.1", 10808);
        assert_eq!(worker.tag, "in-socks");
        assert_eq!(worker.port, 10808);
        assert!(!worker.is_running());

        assert!(worker.start().is_ok());
        assert!(worker.is_running());

        assert!(worker.close().is_ok());
        assert!(!worker.is_running());
    }
}
