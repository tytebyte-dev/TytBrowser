// Module: testing\scenarios\command_test.rs
// Test Commander service registry and CLI version command

#[cfg(test)]
mod tests {
    use crate::app::commander::{Commander, CommanderOutbound, Service};
    use crate::features::outbound::OutboundHandler;
    use crate::main::version::version;
    use std::sync::Arc;

    struct MockStatsService;
    impl Service for MockStatsService {
        fn service_name(&self) -> &str {
            "StatsService"
        }
    }

    #[test]
    fn test_scenario_version_command() {
        assert!(!version().is_empty());
    }

    #[tokio::test]
    async fn test_commander_service_registration_and_lookup() {
        let commander = Arc::new(Commander::new(
            "commander-in".into(),
            "127.0.0.1:10085".into(),
        ));
        commander.register_service(Arc::new(MockStatsService)).await;

        let svc = commander.get_service("StatsService").await;
        assert!(svc.is_ok());
        assert_eq!(svc.unwrap().service_name(), "StatsService");

        let missing = commander.get_service("UnknownService").await;
        assert!(missing.is_err());
    }

    #[tokio::test]
    async fn test_commander_outbound_handler() {
        let commander = Arc::new(Commander::new("cmd".into(), "127.0.0.1:0".into()));
        let outbound = CommanderOutbound::new("cmd-out", commander);
        assert_eq!(outbound.tag(), "cmd-out");
        assert!(outbound.start().await.is_ok());
    }
}
