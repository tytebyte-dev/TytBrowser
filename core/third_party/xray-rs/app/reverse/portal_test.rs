// Module: app\reverse\portal_test.rs
// 1:1 Rust unit test suite corresponding to Go app\reverse\portal_test.go

#[cfg(test)]
mod tests {
    use super::super::bridge::ReverseBridge;
    use super::super::portal::{ReversePortal, new_static_mux_picker};
    use tokio::io::duplex;

    #[test]
    fn test_static_picker_empty() {
        let picker = new_static_mux_picker().expect("create picker");
        let worker = picker.pick_available();
        assert!(worker.is_err());
    }

    #[tokio::test]
    async fn test_bridge_portal_creation() {
        let b = ReverseBridge::new("bridge-0", "reverse.local");
        let p = ReversePortal::new("portal-0", "reverse.local");
        assert_eq!(b.domain, p.domain);

        assert_eq!(b.active_worker_count().await, 1);
        let worker2 = b.spawn_worker_if_needed().await;
        assert_eq!(b.active_worker_count().await, 2);
        worker2.inc_connections();
        assert_eq!(b.total_connections().await, 1);

        // Test stream dispatch through portal
        let (s1, _s2) = duplex(1024);
        p.dispatch(Box::pin(s1)).await.unwrap();

        let pulled = p.pull_stream().await;
        assert!(pulled.is_some());
    }
}
