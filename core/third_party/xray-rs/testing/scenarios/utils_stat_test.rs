#[cfg(test)]
mod tests {
    use crate::app::stats::Counter;
    use crate::common::utils::{TypedSyncMap, h2_base62_pad};
    use crate::transport::internet::stat::StatStream;
    use std::sync::Arc;
    use tokio::io::duplex;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[test]
    fn test_h2_base62_padding_generation() {
        let pad = h2_base62_pad(10);
        assert!(pad.len() >= 10);
        assert!(pad.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_typed_sync_map_operations() {
        let map = TypedSyncMap::<String, i32>::new();
        assert_eq!(map.len(), 0);

        map.store("alpha".into(), 100);
        map.store("beta".into(), 200);

        assert_eq!(map.load(&"alpha".into()), Some(100));
        assert_eq!(map.len(), 2);

        let deleted = map.delete(&"alpha".into());
        assert_eq!(deleted, Some(100));
        assert_eq!(map.load(&"alpha".into()), None);
        assert_eq!(map.len(), 1);
    }

    #[tokio::test]
    async fn test_stat_stream_byte_counting() {
        let (client_raw, server_raw) = duplex(1024);

        let client_read = Arc::new(Counter::new());
        let client_write = Arc::new(Counter::new());

        let mut client = StatStream::new(
            client_raw,
            Some(client_read.clone()),
            Some(client_write.clone()),
        );

        let mut server = server_raw;

        tokio::spawn(async move {
            let mut buf = vec![0u8; 100];
            let n = server.read(&mut buf).await.unwrap();
            server.write_all(&buf[..n]).await.unwrap();
        });

        client.write_all(b"1234567890").await.unwrap();
        assert_eq!(client_write.value(), 10);

        let mut resp = vec![0u8; 10];
        client.read_exact(&mut resp).await.unwrap();
        assert_eq!(resp, b"1234567890");
        assert_eq!(client_read.value(), 10);
    }
}
