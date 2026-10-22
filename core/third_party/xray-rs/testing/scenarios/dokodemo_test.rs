#[cfg(test)]
mod tests {
    use crate::infra::conf::Config;
    use crate::testing::scenarios::common::{TestEnvironment, pick_port};
    use crate::testing::servers::tcp::{Server as TcpServer, echo_processor};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn test_dokodemo_port_forward() {
        let tcp_server = TcpServer::start(None, Some(echo_processor()), None)
            .await
            .unwrap();
        let target_port = tcp_server.port();

        let mut env = TestEnvironment::new();
        let dokodemo_port = pick_port().await;

        // Dokodemo inbound forwarding straight to target_port
        let cfg: Config = serde_json::from_value(serde_json::json!({
            "inbounds": [{
                "tag": "dokodemo-in",
                "port": dokodemo_port,
                "listen": "127.0.0.1",
                "protocol": "dokodemo-door",
                "settings": {
                    "address": "127.0.0.1",
                    "port": target_port,
                    "network": "tcp"
                }
            }],
            "outbounds": [{ "tag": "direct", "protocol": "freedom" }]
        }))
        .unwrap();
        env.start_node(cfg).await.unwrap();

        // Direct connect to dokodemo port (transparent forward)
        let mut client = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", dokodemo_port))
            .await
            .unwrap();

        let msg = b"Testing Dokodemo Transparent Port Forwarding";
        client.write_all(msg).await.unwrap();

        let mut recv = vec![0u8; msg.len()];
        client.read_exact(&mut recv).await.unwrap();
        assert_eq!(&recv, msg);
    }
}
