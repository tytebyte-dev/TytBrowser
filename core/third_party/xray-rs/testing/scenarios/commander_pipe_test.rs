#[cfg(test)]
mod tests {
    use crate::app::commander::{Commander, Service};
    use crate::common::buf::Buffer;
    use crate::transport::pipe::{PipeOption, new_pipe};
    use std::sync::Arc;

    struct MockStatsService;
    impl Service for MockStatsService {
        fn service_name(&self) -> &str {
            "StatsService"
        }
    }

    #[tokio::test]
    async fn test_commander_service_registration() {
        let commander = Commander::new("api".into(), "127.0.0.1:10085".into());
        assert_eq!(commander.service_count().await, 0);

        commander.register_service(Arc::new(MockStatsService)).await;
        assert_eq!(commander.service_count().await, 1);

        let svc = commander.get_service("StatsService").await.unwrap();
        assert_eq!(svc.service_name(), "StatsService");

        assert!(commander.get_service("NonExistent").await.is_err());
    }

    #[tokio::test]
    async fn test_pipe_read_write_flow() {
        let (reader, writer) = new_pipe(PipeOption {
            limit: -1,
            discard_overflow: false,
        });

        let buf = Buffer::from_bytes(b"hello from pipe stream");
        writer.write_buffer(buf).await.unwrap();

        let mb = reader.read_multi_buffer().await.unwrap();
        assert_eq!(mb.len(), 22);
        assert_eq!(mb.to_vec(), b"hello from pipe stream");
    }
}
