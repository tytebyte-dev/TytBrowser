// Module: common\signal\notifier_test.rs
// 1:1 Rust unit test suite corresponding to Go common\signal\notifier_test.go

#[cfg(test)]
mod tests {
    use super::super::notifier::Notifier;

    #[tokio::test]
    async fn test_notifier_signal() {
        let n = Notifier::new();
        let mut sub = n.subscribe();
        n.signal();
        assert!(sub.recv().await.is_ok());
    }

    #[tokio::test]
    async fn test_notifier_wait() {
        let n = Notifier::new();
        let n_clone = n.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            n_clone.signal();
        });
        n.wait().await;
    }
}
