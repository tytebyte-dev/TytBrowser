#[cfg(test)]
mod tests {
    use crate::infra::conf::Config;
    use crate::testing::scenarios::common::{TestEnvironment, pick_port};
    use crate::testing::servers::tcp::{Server as TcpServer, echo_processor};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_vmess_tcp_relay() {
        let tcp_server = TcpServer::start(None, Some(echo_processor()), None)
            .await
            .unwrap();
        let target_port = tcp_server.port();

        let mut env = TestEnvironment::new();
        let user_id = Uuid::new_v4();

        // VMess Server
        let server_port = pick_port().await;
        let server_cfg: Config = serde_json::from_value(serde_json::json!({
            "inbounds": [{
                "tag": "vmess-in",
                "port": server_port,
                "listen": "127.0.0.1",
                "protocol": "vmess",
                "settings": {
                    "clients": [{ "id": user_id.to_string() }]
                }
            }],
            "outbounds": [{ "tag": "direct", "protocol": "freedom" }]
        }))
        .unwrap();
        env.start_node(server_cfg).await.unwrap();

        // VMess Client: SOCKS5 -> VMess Outbound
        let client_port = pick_port().await;
        let client_cfg: Config = serde_json::from_value(serde_json::json!({
            "inbounds": [{ "tag": "socks-in", "port": client_port, "listen": "127.0.0.1", "protocol": "socks" }],
            "outbounds": [{
                "tag": "proxy",
                "protocol": "vmess",
                "settings": {
                    "vnext": [{
                        "address": "127.0.0.1",
                        "port": server_port,
                        "users": [{ "id": user_id.to_string() }]
                    }]
                }
            }]
        })).unwrap();
        env.start_node(client_cfg).await.unwrap();

        let mut client = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", client_port))
            .await
            .unwrap();
        client.write_all(&[0x05, 0x01, 0x00]).await.unwrap();
        let mut auth_resp = [0u8; 2];
        client.read_exact(&mut auth_resp).await.unwrap();

        let mut req = vec![0x05, 0x01, 0x00, 0x01, 127, 0, 0, 1];
        req.extend_from_slice(&target_port.to_be_bytes());
        client.write_all(&req).await.unwrap();

        let mut connect_resp = [0u8; 10];
        client.read_exact(&mut connect_resp).await.unwrap();
        assert_eq!(connect_resp[1], 0x00);

        let msg = b"Testing VMess Relay in Rust Xray-core";
        client.write_all(msg).await.unwrap();

        let mut recv = vec![0u8; msg.len()];
        client.read_exact(&mut recv).await.unwrap();
        assert_eq!(&recv, msg);
    }
}
