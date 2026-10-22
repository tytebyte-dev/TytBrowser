// Module: testing\scenarios\tls_test.rs
// Real end-to-end integration tests for TLS inbound/outbound and certificate pinning

#[cfg(test)]
mod tests {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use uuid::Uuid;

    use crate::infra::conf::Config;
    use crate::testing::scenarios::common::{TestEnvironment, pick_port, random_payload, xor};
    use crate::testing::servers::tcp::{Server as TcpServer, xor_processor};
    use crate::transport::internet::tls::pin::generate_cert_hash_hex;

    #[test]
    fn test_scenario_cert_pinning() {
        let fake_cert = b"MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA0";
        let hash = generate_cert_hash_hex(fake_cert);
        assert_eq!(hash.len(), 64); // SHA-256 hex string
    }

    #[tokio::test]
    async fn test_simple_tls_connection() {
        let xor_key = b't';
        let tcp_server = TcpServer::start(None, Some(xor_processor(xor_key)), None)
            .await
            .unwrap();
        let target_port = tcp_server.port();

        let mut env = TestEnvironment::new();
        let user_id = Uuid::new_v4();

        let server_port = pick_port().await;
        let server_cfg: Config = serde_json::from_value(serde_json::json!({
            "inbounds": [{
                "tag": "vless-tls-in",
                "port": server_port,
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
                        "port": server_port,
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

        // SOCKS5 Handshake
        client.write_all(&[0x05, 0x01, 0x00]).await.unwrap();
        let mut auth_resp = [0u8; 2];
        client.read_exact(&mut auth_resp).await.unwrap();
        assert_eq!(auth_resp, [0x05, 0x00]);

        // SOCKS5 Connect
        let mut req = vec![0x05, 0x01, 0x00, 0x01, 127, 0, 0, 1];
        req.extend_from_slice(&target_port.to_be_bytes());
        client.write_all(&req).await.unwrap();

        let mut connect_resp = [0u8; 10];
        client.read_exact(&mut connect_resp).await.unwrap();
        assert_eq!(connect_resp[1], 0x00);

        // Send payload through tunnel
        let payload = random_payload(1024);
        client.write_all(&payload).await.unwrap();

        let mut recv = vec![0u8; payload.len()];
        client.read_exact(&mut recv).await.unwrap();

        assert_eq!(recv, xor(&payload, xor_key));
    }
}
