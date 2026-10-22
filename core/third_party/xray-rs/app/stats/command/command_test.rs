// Module: app\\stats\\command\\command_test.rs
// 1:1 Rust unit test suite corresponding to Go app\\stats\\command\\command_test.go

#[cfg(test)]
mod tests {
    use super::super::command::StatsCommandServer;
    use crate::app::stats::StatsManager;
    use std::sync::Arc;

    #[test]
    fn test_stats_service_query() {
        let sm = Arc::new(StatsManager::new());
        let c1 = sm.register_counter_sync("inbound>>>in-1>>>traffic>>>downlink");
        c1.add(500);
        let c2 = sm.register_counter_sync("outbound>>>out-1>>>traffic>>>uplink");
        c2.add(100);

        let svc = StatsCommandServer::new(sm);
        assert_eq!(
            svc.get_stat_value("inbound>>>in-1>>>traffic>>>downlink"),
            500
        );
        let queried = svc.query_stats("traffic>>>downlink");
        assert_eq!(queried.len(), 1);
        assert_eq!(
            queried.get("inbound>>>in-1>>>traffic>>>downlink"),
            Some(&500)
        );
    }
}
