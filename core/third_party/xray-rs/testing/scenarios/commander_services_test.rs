#[cfg(test)]
mod tests {
    use crate::app::commander::Service;
    use crate::app::log::command::LoggerService;
    use crate::app::log::{LogLevel, LogManager};
    use crate::app::observatory::Observatory;
    use crate::app::observatory::command::ObservatoryService;
    use crate::app::proxyman::command::HandlerService;
    use crate::app::proxyman::inbound::DefaultInboundManager;
    use crate::app::proxyman::outbound::DefaultOutboundManager;
    use crate::app::router::Router;
    use crate::app::router::command::RoutingService;
    use crate::app::stats::StatsManager;
    use crate::app::stats::command::StatsService;
    use std::sync::Arc;
    use std::time::Duration;

    #[tokio::test]
    async fn test_all_commander_services() {
        // 1. Logger Service
        let log_mgr = Arc::new(LogManager::new(LogLevel::Info, 24, 64));
        let logger_svc = LoggerService::new(log_mgr);
        assert_eq!(
            logger_svc.service_name(),
            "xray.core.app.log.command.LoggerService"
        );
        assert!(logger_svc.restart_logger().is_ok());

        // 2. Stats Service
        let stats_mgr = Arc::new(StatsManager::new());
        let counter = stats_mgr.register_counter_sync("inbound>>>direct>>>traffic>>>downlink");
        counter.add(1024);
        let stats_svc = StatsService::new(stats_mgr);
        assert_eq!(
            stats_svc.service_name(),
            "xray.core.app.stats.command.StatsService"
        );
        assert_eq!(
            stats_svc.get_stat("inbound>>>direct>>>traffic>>>downlink", false),
            1024
        );
        let queried = stats_svc.query_stats("direct", false);
        assert_eq!(
            queried.get("inbound>>>direct>>>traffic>>>downlink"),
            Some(&1024)
        );

        // 3. Handler Service
        let inbound_mgr = Arc::new(DefaultInboundManager::new());
        let outbound_mgr = Arc::new(DefaultOutboundManager::new());
        let handler_svc = HandlerService::new(Some(inbound_mgr), Some(outbound_mgr));
        assert_eq!(
            handler_svc.service_name(),
            "xray.core.app.proxyman.command.HandlerService"
        );
        assert!(!handler_svc.remove_inbound("non_existent").await.unwrap());

        // 4. Routing Service
        let router = Arc::new(Router::new(Vec::new(), None));
        let routing_svc = RoutingService::new(router);
        assert_eq!(
            routing_svc.service_name(),
            "xray.core.app.router.command.RoutingService"
        );

        // 5. Observatory Service
        let obs = Arc::new(Observatory::new(
            "http://www.google.com/gen_204",
            Duration::from_secs(10),
            vec![],
        ));
        obs.record_status("direct", true, 45, None);
        let obs_svc = ObservatoryService::new(obs);
        assert_eq!(
            obs_svc.service_name(),
            "xray.core.app.observatory.command.ObservatoryService"
        );
        let st = obs_svc.get_outbound_status("direct").unwrap();
        assert!(st.alive);
        assert_eq!(st.delay_ms, 45);
    }
}
