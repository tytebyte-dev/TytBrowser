// Module: transport\internet\splithttp\splithttp_test.rs
// 1:1 Rust unit test suite corresponding to Go transport\internet\splithttp\splithttp_test.go

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::net::SocketAddr;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use crate::common::net::{Address, Destination};
    use crate::transport::internet::dialer::Dialer;
    use crate::transport::internet::splithttp::UploadQueue;
    use crate::transport::internet::splithttp::common::*;
    use crate::transport::internet::splithttp::config::SplitHttpConfig;
    use crate::transport::internet::splithttp::dialer::SplitHttpDialer;
    use crate::transport::internet::splithttp::h1_conn::H1Conn;
    use crate::transport::internet::splithttp::hub::SplitHttpHub;

    #[tokio::test]
    async fn test_splithttp_pipeline() {
        let queue = UploadQueue::new(5);
        let res = queue.push_bytes(vec![1, 2, 3]).await;
        assert!(res.is_ok());
        let popped = queue.pop().await;
        assert_eq!(popped, Some(vec![1, 2, 3]));
    }

    #[tokio::test]
    async fn test_h1_conn_status_read() {
        let (mut client, server) = tokio::io::duplex(1024);
        let mut h1 = H1Conn::new(Box::pin(server));

        tokio::spawn(async move {
            let _ = client
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
                .await;
        });

        h1.inc_unread();
        let status = h1.read_response_status().await.unwrap();
        assert_eq!(status, 200);
        assert_eq!(h1.unread_responses_count, 0);
    }

    #[test]
    fn test_config_normalization() {
        let cfg = SplitHttpConfig::new("test-path?param=1", "example.com");
        assert_eq!(cfg.get_normalized_path(), "/test-path/");
        assert_eq!(cfg.get_normalized_query(), "param=1");
        assert_eq!(cfg.get_normalized_session_placement(), PLACEMENT_PATH);
        assert_eq!(cfg.get_normalized_seq_placement(), PLACEMENT_PATH);
        assert_eq!(cfg.get_normalized_uplink_http_method(), "POST");

        let (sid, seq) = cfg.extract_meta_from_request(
            "/test-path/session-abc/123",
            "/test-path/",
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert_eq!(sid, "session-abc");
        assert_eq!(seq, "123");
    }

    #[tokio::test]
    async fn test_splithttp_end_to_end() {
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let mut cfg = SplitHttpConfig::new("/splithttp-test/", "127.0.0.1");
        cfg.sc_max_each_post_bytes =
            Some(crate::transport::internet::splithttp::config::RangeConfig::new(1024, 1024));

        let hub = SplitHttpHub::listen(addr, cfg.clone()).await.unwrap();
        let local_addr = hub.local_addr().unwrap();

        // Hub accept task
        tokio::spawn(async move {
            if let Ok((mut stream, _)) = hub.accept().await {
                let mut buf = vec![0u8; 12];
                let _ = stream.read_exact(&mut buf).await;
                let _ = stream.write_all(b"hello client").await;
                let _ = stream.flush().await;
            }
        });

        // Dialer
        let dialer = SplitHttpDialer::new(cfg);
        let dest = Destination::tcp(Address::ip(local_addr.ip()), local_addr.port());

        let mut client_stream = dialer.dial(&dest).await.unwrap();
        client_stream.write_all(b"hello server").await.unwrap();
        client_stream.flush().await.unwrap();

        let mut resp = vec![0u8; 12];
        client_stream.read_exact(&mut resp).await.unwrap();
        assert_eq!(&resp, b"hello client");
    }
}
