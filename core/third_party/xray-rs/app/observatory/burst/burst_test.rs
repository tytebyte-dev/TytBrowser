// Module: app\observatory\burst\burst_test.rs
// 1:1 Rust unit test suite corresponding to Go app\observatory\burst\burst_test.go

#[cfg(test)]
mod tests {
    use super::super::health::HealthStatus;
    use super::super::observer::BurstObserver;
    use super::super::selector::FastSelector;

    #[tokio::test]
    async fn test_burst_observer_and_selector() {
        let observer = BurstObserver::new();
        observer
            .update(HealthStatus::new("us-east", true, 120))
            .await;
        observer
            .update(HealthStatus::new("us-west", true, 60))
            .await;
        observer
            .update(HealthStatus::new("eu-central", false, 0))
            .await;

        let s1 = observer.get("us-west").await.unwrap();
        assert_eq!(s1.latency_ms, 60);

        let list = vec![
            observer.get("us-east").await.unwrap(),
            observer.get("us-west").await.unwrap(),
            observer.get("eu-central").await.unwrap(),
        ];

        let best = FastSelector::select_fastest(&list).unwrap();
        assert_eq!(best.tag, "us-west");
    }
}
