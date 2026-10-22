// Module: app\proxyman\inbound\inbound_test.rs
// 1:1 Rust unit test suite corresponding to Go app\proxyman\inbound\always_test.go

#[cfg(test)]
mod tests {
    use super::super::{AlwaysOnInboundHandler, InboundWorker, always::ALWAYS_ON};
    use crate::app::stats::StatsManager;
    use crate::features::inbound::InboundHandler;

    #[tokio::test]
    async fn test_always_on_inbound_handler_lifecycle() {
        assert!(ALWAYS_ON);
        let stats_mgr = StatsManager::new();
        let handler = AlwaysOnInboundHandler::new(
            "socks-in",
            Some(&stats_mgr),
            Some(vec![1, 2, 3]),
            Some(vec![4, 5, 6]),
        );

        assert_eq!(handler.tag(), "socks-in");
        assert_eq!(handler.receiver_settings(), Some(vec![1, 2, 3]));
        assert_eq!(handler.proxy_settings(), Some(vec![4, 5, 6]));

        // Check stats counters
        assert!(handler.uplink_counter().is_some());
        assert!(handler.downlink_counter().is_some());

        let up = handler.uplink_counter().unwrap();
        up.add(1024);
        assert_eq!(up.value(), 1024);

        // Add workers
        let worker1 = InboundWorker::new("socks-in", "127.0.0.1", 1080);
        let worker2 = InboundWorker::new("socks-in", "127.0.0.1", 1081);
        handler.add_worker(worker1).await;
        handler.add_worker(worker2).await;

        // Start and close
        handler.start().await.expect("start handler");
        handler.close().await.expect("close handler");
    }
}
