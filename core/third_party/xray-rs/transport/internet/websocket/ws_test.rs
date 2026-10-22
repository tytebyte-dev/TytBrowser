// Module: transport\internet\websocket\ws_test.rs
// 1:1 Rust unit test suite corresponding to Go transport\internet\websocket\ws_test.go

#[cfg(test)]
mod tests {
    use super::super::config::WebSocketConfig;
    use super::super::dialer::WebSocketDialer;
    use super::super::hub::WebSocketHub;
    use crate::common::net::{Address, Destination};
    use std::net::SocketAddr;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn test_listen_ws_and_dial_roundtrip() {
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let hub = WebSocketHub::listen(addr)
            .await
            .expect("Listen on local port");
        let local_addr = hub.local_addr();

        let server_task = tokio::spawn(async move {
            let (mut stream, _) = hub.accept().await.expect("Accept WS connection");
            let mut buf = vec![0u8; 128];
            let n = stream.read(&mut buf).await.expect("Read request");
            let req = String::from_utf8_lossy(&buf[..n]);
            assert!(req.starts_with("Test connection"));

            stream.write_all(b"Response").await.expect("Write response");
            stream.flush().await.expect("Flush response");
        });

        let config = WebSocketConfig {
            path: "ws".into(),
            ..Default::default()
        };
        let dest = Destination::tcp(Address::ip(local_addr.ip()), local_addr.port());

        let mut client_stream = WebSocketDialer::dial(&dest, &config)
            .await
            .expect("Dial WS server");
        client_stream
            .write_all(b"Test connection 1")
            .await
            .expect("Write request");
        client_stream.flush().await.expect("Flush request");

        let mut resp_buf = vec![0u8; 128];
        let n = client_stream
            .read(&mut resp_buf)
            .await
            .expect("Read response");
        assert_eq!(&resp_buf[..n], b"Response");

        server_task.await.expect("Server task should complete");
    }

    #[tokio::test]
    async fn test_ws_multiple_frames_flow() {
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let hub = WebSocketHub::listen(addr).await.unwrap();
        let local_addr = hub.local_addr();

        let server_task = tokio::spawn(async move {
            let (mut stream, _) = hub.accept().await.unwrap();
            for i in 0..5 {
                let mut buf = vec![0u8; 64];
                let n = stream.read(&mut buf).await.unwrap();
                let expected = format!("frame-{}", i);
                assert_eq!(&buf[..n], expected.as_bytes());

                let reply = format!("ack-{}", i);
                stream.write_all(reply.as_bytes()).await.unwrap();
                stream.flush().await.unwrap();
            }
        });

        let config = WebSocketConfig {
            path: "test".into(),
            ..Default::default()
        };
        let dest = Destination::tcp(Address::ip(local_addr.ip()), local_addr.port());
        let mut client = WebSocketDialer::dial(&dest, &config).await.unwrap();

        for i in 0..5 {
            let msg = format!("frame-{}", i);
            client.write_all(msg.as_bytes()).await.unwrap();
            client.flush().await.unwrap();

            let mut buf = vec![0u8; 64];
            let n = client.read(&mut buf).await.unwrap();
            let expected_ack = format!("ack-{}", i);
            assert_eq!(&buf[..n], expected_ack.as_bytes());
        }

        server_task.await.unwrap();
    }
}
