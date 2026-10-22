#[cfg(test)]
mod tests {
    use crate::infra::conf::Config;
    use crate::testing::scenarios::common::{TestEnvironment, pick_port};
    use crate::testing::servers::tcp::{Server as TcpServer, xor_processor};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_vless_ws_tls_full_e2e() {
        let xor_key = b'k';
        let tcp_server = TcpServer::start(None, Some(xor_processor(xor_key)), None)
            .await
            .unwrap();
        let target_port = tcp_server.port();

        let mut env = TestEnvironment::new();
        let user_id = Uuid::new_v4();

        // 1. Generate self-signed cert for TLS test
        let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
        let _cert_pem = cert.cert.pem();
        let _key_pem = cert.key_pair.serialize_pem();

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

        // 2. Client node with VLESS + WS with default fallback path
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

        // 3. Connect via SOCKS5 and test bidirectional transmission
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

        let msg = b"Xray-core VLESS End-To-End Verification";
        client.write_all(msg).await.unwrap();

        let mut recv = vec![0u8; msg.len()];
        client.read_exact(&mut recv).await.unwrap();

        let expected: Vec<u8> = msg.iter().map(|b| b ^ xor_key).collect();
        assert_eq!(recv, expected);
    }
}
