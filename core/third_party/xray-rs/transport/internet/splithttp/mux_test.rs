// Module: transport\internet\splithttp\mux_test.rs
// 1:1 Rust unit test suite corresponding to Go transport\internet\splithttp\mux_test.go

#[cfg(test)]
mod tests {
    use super::super::mux::XmuxManager;

    #[tokio::test]
    async fn test_splithttp_mux_lifecycle() {
        let mut mux = XmuxManager::new(2, 2, 5, 10, 60);

        let client1 = mux.get_client().await;
        {
            let guard = client1.lock().await;
            assert!(guard.acquire());
            assert_eq!(
                guard.open_usage.load(std::sync::atomic::Ordering::SeqCst),
                1
            );
            guard.release();
        }

        let client2 = mux.get_client().await;
        let c1_id = client1.lock().await.id;
        let c2_id = client2.lock().await.id;
        // Same client reused while under max_concurrency
        assert_eq!(c1_id, c2_id);
    }
}
