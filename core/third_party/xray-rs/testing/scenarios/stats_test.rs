#[cfg(test)]
mod tests {
    use crate::app::stats::{Counter, StatsManager};

    #[test]
    fn test_atomic_counter_operations() {
        let counter = Counter::new();
        assert_eq!(counter.value(), 0);

        assert_eq!(counter.add(100), 100);
        assert_eq!(counter.value(), 100);

        assert_eq!(counter.add(50), 150);
        assert_eq!(counter.value(), 150);

        let old = counter.set(0);
        assert_eq!(old, 150);
        assert_eq!(counter.value(), 0);
    }

    #[tokio::test]
    async fn test_stats_manager_registry() {
        let mgr = StatsManager::new();
        let uplink = mgr
            .register_counter("inbound>>>proxy>>>traffic>>>uplink")
            .await;
        let downlink = mgr
            .register_counter("inbound>>>proxy>>>traffic>>>downlink")
            .await;

        uplink.add(1024);
        downlink.add(2048);

        let all_stats = mgr.get_all_stats().await;
        assert_eq!(
            all_stats.get("inbound>>>proxy>>>traffic>>>uplink"),
            Some(&1024)
        );
        assert_eq!(
            all_stats.get("inbound>>>proxy>>>traffic>>>downlink"),
            Some(&2048)
        );
    }
}
