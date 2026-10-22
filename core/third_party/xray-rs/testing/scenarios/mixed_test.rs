#[cfg(test)]
mod tests {
    use crate::infra::conf::Config;
    use crate::testing::scenarios::common::{TestEnvironment, pick_port};
    use crate::testing::servers::tcp::{Server as TcpServer, echo_processor};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn test_mixed_inbound_socks_and_http_dual_mode() {
        let tcp_server = TcpServer::start(None, Some(echo_processor()), None)
            .await
            .unwrap();
        let target_port = tcp_server.port();

        let mut env = TestEnvironment::new();
        let mixed_port = pick_port().await;

        let cfg: Config = serde_json::from_value(serde_json::json!({
            "inbounds": [{
                "tag": "mixed-in",
                "port": mixed_port,
                "listen": "127.0.0.1",
                "protocol": "mixed"
            }],
            "outbounds": [{ "tag": "direct", "protocol": "freedom" }]
        }))
        .unwrap();
        env.start_node(cfg).await.unwrap();

        // 1. Test SOCKS5 mode on mixed port
        let mut socks_client = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", mixed_port))
            .await
            .unwrap();
        socks_client.write_all(&[0x05, 0x01, 0x00]).await.unwrap();
        let mut auth_resp = [0u8; 2];
        socks_client.read_exact(&mut auth_resp).await.unwrap();
        assert_eq!(auth_resp, [0x05, 0x00]);

        let mut req = vec![0x05, 0x01, 0x00, 0x01, 127, 0, 0, 1];
        req.extend_from_slice(&target_port.to_be_bytes());
        socks_client.write_all(&req).await.unwrap();

        let mut connect_resp = [0u8; 10];
        socks_client.read_exact(&mut connect_resp).await.unwrap();
        assert_eq!(connect_resp[1], 0x00);

        let msg1 = b"Hello from SOCKS5 on Mixed Port";
        socks_client.write_all(msg1).await.unwrap();
        let mut recv1 = vec![0u8; msg1.len()];
        socks_client.read_exact(&mut recv1).await.unwrap();
        assert_eq!(&recv1, msg1);

        // 2. Test HTTP CONNECT mode on the exact same mixed port
        let mut http_client = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", mixed_port))
            .await
            .unwrap();
        let connect_req = format!(
            "CONNECT 127.0.0.1:{} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\n\r\n",
            target_port, target_port
        );
        http_client.write_all(connect_req.as_bytes()).await.unwrap();

        let mut http_resp_buf = [0u8; 1024];
        let n = http_client.read(&mut http_resp_buf).await.unwrap();
        let http_resp_str = String::from_utf8_lossy(&http_resp_buf[..n]);
        assert!(http_resp_str.contains("200 Connection Established"));

        let msg2 = b"Hello from HTTP CONNECT on Mixed Port";
        http_client.write_all(msg2).await.unwrap();
        let mut recv2 = vec![0u8; msg2.len()];
        http_client.read_exact(&mut recv2).await.unwrap();
        assert_eq!(&recv2, msg2);
    }
}
