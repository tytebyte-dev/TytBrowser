// Module: transport\internet\httpupgrade\httpupgrade_test.rs
// 1:1 Rust unit test suite corresponding to Go transport\internet\httpupgrade\httpupgrade_test.go

#[cfg(test)]
mod tests {
    use super::super::config::HttpUpgradeConfig;
    use super::super::dialer::HttpUpgradeDialer;
    use super::super::httpupgrade::{HttpUpgradeStream, PROTOCOL_NAME, UpgradedStream};
    use super::super::hub::HttpUpgradeHub;
    use crate::common::net::{Address, Destination};
    use std::collections::HashMap;
    use std::net::SocketAddr;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[test]
    fn test_httpupgrade_constants_and_normalized_path() {
        assert_eq!(PROTOCOL_NAME, "httpupgrade");

        let cfg1 = HttpUpgradeConfig::new("example.com", "");
        assert_eq!(cfg1.get_normalized_path(), "/");

        let cfg2 = HttpUpgradeConfig::new("example.com", "vless-upgrade");
        assert_eq!(cfg2.get_normalized_path(), "/vless-upgrade");

        let cfg3 = HttpUpgradeConfig::new("example.com", "/already/slash");
        assert_eq!(cfg3.get_normalized_path(), "/already/slash");
    }

    #[tokio::test]
    async fn test_httpupgrade_handshake_memory_duplex() {
        let (client_raw, server_raw) = tokio::io::duplex(4096);

        let server_task = tokio::spawn(async move {
            let mut server_stream = HttpUpgradeStream::server_handshake(
                Box::pin(server_raw),
                Some("example.com"),
                Some("/upgrade"),
            )
            .await
            .expect("server handshake should succeed");

            let mut buf = [0u8; 128];
            let n = server_stream.read(&mut buf).await.expect("server read");
            assert_eq!(&buf[..n], b"Ping HttpUpgrade");

            server_stream
                .write_all(b"Pong HttpUpgrade")
                .await
                .expect("server write");
            server_stream.flush().await.expect("server flush");
        });

        let client_task = tokio::spawn(async move {
            let mut client_stream = HttpUpgradeStream::client_handshake(
                "example.com",
                "/upgrade",
                Box::pin(client_raw),
                &HashMap::new(),
            )
            .await
            .expect("client handshake should succeed");

            client_stream
                .write_all(b"Ping HttpUpgrade")
                .await
                .expect("client write");
            client_stream.flush().await.expect("client flush");

            let mut buf = [0u8; 128];
            let n = client_stream.read(&mut buf).await.expect("client read");
            assert_eq!(&buf[..n], b"Pong HttpUpgrade");
        });

        let (s_res, c_res) = tokio::join!(server_task, client_task);
        s_res.expect("server task join");
        c_res.expect("client task join");
    }

    #[tokio::test]
    async fn test_httpupgrade_pipelined_prefixed_data() {
        let (c_raw, s_raw) = tokio::io::duplex(4096);

        let upgraded_server = UpgradedStream::new(Box::pin(s_raw), b"Prefixed Early Data".to_vec());
        let mut server_stream: crate::common::net::BoxStream = Box::pin(upgraded_server);

        let mut client_stream = Box::pin(c_raw);

        // Server should first yield the prefixed data
        let mut read_buf = [0u8; 19];
        server_stream
            .read_exact(&mut read_buf)
            .await
            .expect("should read prefixed data");
        assert_eq!(&read_buf, b"Prefixed Early Data");

        // Subsequent read comes from stream
        client_stream
            .write_all(b"Stream Data")
            .await
            .expect("client write");
        client_stream.flush().await.expect("client flush");

        let mut read_buf2 = [0u8; 11];
        server_stream
            .read_exact(&mut read_buf2)
            .await
            .expect("should read stream data");
        assert_eq!(&read_buf2, b"Stream Data");
    }

    #[tokio::test]
    async fn test_httpupgrade_path_mismatch_rejected() {
        let (client_raw, server_raw) = tokio::io::duplex(4096);

        let server_task = tokio::spawn(async move {
            let res = HttpUpgradeStream::server_handshake(
                Box::pin(server_raw),
                Some("example.com"),
                Some("/expected_path"),
            )
            .await;
            assert!(res.is_err());
        });

        let client_task = tokio::spawn(async move {
            let _ = HttpUpgradeStream::client_handshake(
                "example.com",
                "/wrong_path",
                Box::pin(client_raw),
                &HashMap::new(),
            )
            .await;
        });

        let _ = tokio::join!(server_task, client_task);
    }

    #[tokio::test]
    async fn test_httpupgrade_hub_dialer_end_to_end() {
        let bind_addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let config = HttpUpgradeConfig::new("localhost", "/xray-tunnel");
        let hub = HttpUpgradeHub::listen(bind_addr, config.clone())
            .await
            .expect("hub listen failed");
        let local_addr = hub.local_addr();

        let server_task = tokio::spawn(async move {
            let conn = hub.accept().await.expect("hub accept failed");
            let mut stream = conn.into_inner();

            let mut buf = [0u8; 64];
            let n = stream.read(&mut buf).await.expect("server read");
            assert_eq!(&buf[..n], b"Client Payload");

            stream
                .write_all(b"Server Reply")
                .await
                .expect("server write");
            stream.flush().await.expect("server flush");
        });

        let client_task = tokio::spawn(async move {
            let dest = Destination {
                network: crate::common::net::Network::Tcp,
                address: Address::ip(local_addr.ip()),
                port: local_addr.port(),
            };

            let mut stream = HttpUpgradeDialer::dial(&dest, &config)
                .await
                .expect("dial failed");

            stream
                .write_all(b"Client Payload")
                .await
                .expect("client write");
            stream.flush().await.expect("client flush");

            let mut buf = [0u8; 64];
            let n = stream.read(&mut buf).await.expect("client read");
            assert_eq!(&buf[..n], b"Server Reply");
        });

        let (s_res, c_res) = tokio::join!(server_task, client_task);
        s_res.expect("server join");
        c_res.expect("client join");
    }
}
