// Module: common\signal\pubsub\pubsub_test.rs
// 1:1 Rust unit test suite corresponding to Go common\signal\pubsub\pubsub_test.go

#[cfg(test)]
mod tests {
    use super::super::pubsub::Service;

    #[tokio::test]
    async fn test_pubsub() {
        let service = Service::new(16);
        let mut sub = service.subscribe("a");
        service.publish("a", "1").unwrap();

        assert_eq!(sub.try_recv(), Some("1".to_string()));

        sub.close().unwrap();
        service.publish("a", "2").unwrap();

        let _ = service.cleanup();
    }
}
