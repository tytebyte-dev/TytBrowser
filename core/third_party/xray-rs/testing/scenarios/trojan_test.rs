#[cfg(test)]
mod tests {
    use crate::infra::conf::Config;
    use crate::testing::scenarios::common::{TestEnvironment, pick_port};
    use crate::testing::servers::tcp::{Server as TcpServer, echo_processor};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn test_trojan_tcp_auth() {
        let tcp_server = TcpServer::start(None, Some(echo_processor()), None)
            .await
            .unwrap();
        let target_port = tcp_server.port();

        let mut env = TestEnvironment::new();
        let password = "my_secure_trojan_password";

        // Trojan Server
        let server_port = pick_port().await;
        let server_cfg: Config = serde_json::from_value(serde_json::json!({
            "inbounds": [{
                "tag": "trojan-in",
                "port": server_port,
                "listen": "127.0.0.1",
                "protocol": "trojan",
                "settings": {
                    "clients": [{ "id": "user1", "password": password }]
                }
            }],
            "outbounds": [{ "tag": "direct", "protocol": "freedom" }]
        }))
        .unwrap();
        env.start_node(server_cfg).await.unwrap();

        // Client: SOCKS5 -> Trojan Outbound
        let client_port = pick_port().await;
        let client_cfg: Config = serde_json::from_value(serde_json::json!({
            "inbounds": [{ "tag": "socks-in", "port": client_port, "listen": "127.0.0.1", "protocol": "socks" }],
            "outbounds": [{
                "tag": "proxy",
                "protocol": "trojan",
                "settings": {
                    "servers": [{
                        "address": "127.0.0.1",
                        "port": server_port,
                        "password": password
                    }]
                }
            }]
        })).unwrap();
        env.start_node(client_cfg).await.unwrap();

        // Connect via SOCKS5
        let mut client = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", client_port))
            .await
            .unwrap();
        client.write_all(&[0x05, 0x01, 0x00]).await.unwrap();
        let mut auth_resp = [0u8; 2];
        client.read_exact(&mut auth_resp).await.unwrap();
        assert_eq!(auth_resp, [0x05, 0x00]);

        let mut req = vec![0x05, 0x01, 0x00, 0x01, 127, 0, 0, 1];
        req.extend_from_slice(&target_port.to_be_bytes());
        client.write_all(&req).await.unwrap();

        let mut connect_resp = [0u8; 10];
        client.read_exact(&mut connect_resp).await.unwrap();
        assert_eq!(connect_resp[1], 0x00);

        let msg = b"Testing Trojan Relay in Rust Xray-core";
        client.write_all(msg).await.unwrap();

        let mut recv = vec![0u8; msg.len()];
        client.read_exact(&mut recv).await.unwrap();
        assert_eq!(&recv, msg);
    }
}
