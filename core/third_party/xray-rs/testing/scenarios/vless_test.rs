#[cfg(test)]
mod tests {
    use crate::infra::conf::Config;
    use crate::testing::scenarios::common::{TestEnvironment, pick_port, random_payload};
    use crate::testing::servers::tcp::{Server as TcpServer, xor_processor};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_vless_tcp_xor() {
        let xor_key = b'c';
        let tcp_server = TcpServer::start(None, Some(xor_processor(xor_key)), None)
            .await
            .unwrap();
        let target_port = tcp_server.port();

        let mut env = TestEnvironment::new();
        let user_id = Uuid::new_v4();

        let server_vless_port = pick_port().await;
        let server_cfg: Config = serde_json::from_value(serde_json::json!({
            "inbounds": [{
                "tag": "vless-in",
                "port": server_vless_port,
                "listen": "127.0.0.1",
                "protocol": "vless",
                "settings": {
                    "clients": [{ "id": user_id.to_string() }]
                }
            }],
            "outbounds": [{
                "tag": "direct",
                "protocol": "freedom"
            }]
        }))
        .unwrap();
        env.start_node(server_cfg).await.unwrap();

        let client_socks_port = pick_port().await;
        let client_cfg: Config = serde_json::from_value(serde_json::json!({
            "inbounds": [{
                "tag": "socks-in",
                "port": client_socks_port,
                "listen": "127.0.0.1",
                "protocol": "socks"
            }],
            "outbounds": [{
                "tag": "proxy",
                "protocol": "vless",
                "settings": {
                    "vnext": [{
                        "address": "127.0.0.1",
                        "port": server_vless_port,
                        "users": [{ "id": user_id.to_string() }]
                    }]
                }
            }]
        }))
        .unwrap();
        env.start_node(client_cfg).await.unwrap();

        let mut client = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", client_socks_port))
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

        let msg = b"Xray-core VLESS XOR Test Message";
        client.write_all(msg).await.unwrap();

        let mut recv = vec![0u8; msg.len()];
        client.read_exact(&mut recv).await.unwrap();

        let expected: Vec<u8> = msg.iter().map(|b| b ^ xor_key).collect();
        assert_eq!(recv, expected);
    }

    #[tokio::test]
    async fn test_vless_unauthorized_user_rejected() {
        let tcp_server = TcpServer::start(None, None, None).await.unwrap();
        let target_port = tcp_server.port();

        let mut env = TestEnvironment::new();
        let valid_user = Uuid::new_v4();
        let invalid_user = Uuid::new_v4();

        let server_port = pick_port().await;
        let server_cfg: Config = serde_json::from_value(serde_json::json!({
            "inbounds": [{
                "tag": "vless-in",
                "port": server_port,
                "listen": "127.0.0.1",
                "protocol": "vless",
                "settings": { "clients": [{ "id": valid_user.to_string() }] }
            }],
            "outbounds": [{ "tag": "direct", "protocol": "freedom" }]
        }))
        .unwrap();
        env.start_node(server_cfg).await.unwrap();

        let client_port = pick_port().await;
        let client_cfg: Config = serde_json::from_value(serde_json::json!({
            "inbounds": [{ "tag": "socks-in", "port": client_port, "listen": "127.0.0.1", "protocol": "socks" }],
            "outbounds": [{
                "tag": "proxy",
                "protocol": "vless",
                "settings": {
                    "vnext": [{
                        "address": "127.0.0.1",
                        "port": server_port,
                        "users": [{ "id": invalid_user.to_string() }]
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

        client.write_all(b"Ping").await.unwrap();
        let mut buf = [0u8; 16];
        let n = client.read(&mut buf).await.unwrap_or(0);
        assert_eq!(
            n, 0,
            "Unauthorized connection must be closed by VLESS server"
        );
    }

    #[tokio::test]
    async fn test_vless_large_payload_stream() {
        let tcp_server = TcpServer::start(None, None, None).await.unwrap();
        let target_port = tcp_server.port();

        let mut env = TestEnvironment::new();
        let user_id = Uuid::new_v4();
        let server_port = pick_port().await;
        let client_port = pick_port().await;

        env.start_node(serde_json::from_value(serde_json::json!({
            "inbounds": [{ "tag": "vless-in", "port": server_port, "listen": "127.0.0.1", "protocol": "vless", "settings": { "clients": [{ "id": user_id.to_string() }] } }],
            "outbounds": [{ "tag": "direct", "protocol": "freedom" }]
        })).unwrap()).await.unwrap();

        env.start_node(serde_json::from_value(serde_json::json!({
            "inbounds": [{ "tag": "socks-in", "port": client_port, "listen": "127.0.0.1", "protocol": "socks" }],
            "outbounds": [{ "tag": "proxy", "protocol": "vless", "settings": { "vnext": [{ "address": "127.0.0.1", "port": server_port, "users": [{ "id": user_id.to_string() }] }] } }]
        })).unwrap()).await.unwrap();

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

        let large_payload = random_payload(256 * 1024);
        let (mut read_half, mut write_half) = client.into_split();

        let send_payload = large_payload.clone();
        let write_task = tokio::spawn(async move {
            write_half.write_all(&send_payload).await.unwrap();
        });

        let mut recv_buf = vec![0u8; large_payload.len()];
        read_half.read_exact(&mut recv_buf).await.unwrap();

        write_task.await.unwrap();
        assert_eq!(recv_buf, large_payload);
    }
}
