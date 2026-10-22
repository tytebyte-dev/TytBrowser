// Module: app\commander\commander_test.rs

#[cfg(test)]
mod tests {
    use crate::app::commander::{Commander, CommanderOutbound, Service};
    use crate::common::net::{Address, Destination, Network};
    use crate::common::protocol::SessionContext;
    use crate::features::outbound::OutboundHandler;
    use std::sync::Arc;

    struct DummyService(&'static str);
    impl Service for DummyService {
        fn service_name(&self) -> &str {
            self.0
        }
    }

    #[tokio::test]
    async fn test_commander_lifecycle_and_services() {
        let cmd = Arc::new(Commander::new("api".into(), "127.0.0.1:10085".into()));
        assert_eq!(cmd.tag, "api");
        assert_eq!(cmd.listen, "127.0.0.1:10085");
        assert_eq!(cmd.service_count().await, 0);

        let svc1 = Arc::new(DummyService("StatsService"));
        cmd.register_service(svc1).await;
        assert_eq!(cmd.service_count().await, 1);

        let svc_retrieved = cmd.get_service("StatsService").await;
        assert!(svc_retrieved.is_ok());
        assert_eq!(svc_retrieved.unwrap().service_name(), "StatsService");

        assert!(cmd.get_service("NonExistent").await.is_err());

        // Outbound wrapper
        let outbound = CommanderOutbound::new("api_out", cmd.clone());
        assert_eq!(outbound.tag(), "api_out");
        assert!(outbound.start().await.is_ok());

        let sess = SessionContext::new(
            "api_in",
            Destination {
                network: Network::Tcp,
                address: Address::domain("api.internal"),
                port: 10085,
            },
        );
        let stream = outbound.connect(&sess).await;
        assert!(stream.is_ok());
        assert!(outbound.close().await.is_ok());
    }
}
