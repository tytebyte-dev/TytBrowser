// Module: testing\scenarios\socks_test.rs
// Real end-to-end integration tests for SOCKS5 proxy protocol

#[cfg(test)]
mod tests {
    use crate::infra::conf::Config;
    use crate::testing::scenarios::common::{
        TestEnvironment, pick_port, random_payload, socks5_connect, xor,
    };
    use crate::testing::servers::http::Server as HttpServer;
    use crate::testing::servers::tcp::{Server as TcpServer, echo_processor, xor_processor};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn test_socks5_direct_echo() {
        let tcp_server = TcpServer::start(None, Some(echo_processor()), None)
            .await
            .unwrap();
        let target_port = tcp_server.port();

        let mut env = TestEnvironment::new();
        let socks_port = pick_port().await;

        let cfg: Config = serde_json::from_value(serde_json::json!({
            "inbounds": [{
                "tag": "socks-in",
                "port": socks_port,
                "listen": "127.0.0.1",
                "protocol": "socks"
            }],
            "outbounds": [{
                "tag": "direct",
                "protocol": "freedom"
            }]
        }))
        .unwrap();
        env.start_node(cfg).await.unwrap();

        let mut client = socks5_connect(socks_port, "localhost", target_port)
            .await
            .unwrap();

        let msg = b"Testing SOCKS5 Direct Domain Resolution";
        client.write_all(msg).await.unwrap();

        let mut recv = vec![0u8; msg.len()];
        client.read_exact(&mut recv).await.unwrap();
        assert_eq!(&recv, msg);
    }

    #[tokio::test]
    async fn test_socks5_http_request() {
        // 1. Start HTTP target server
        let http_server = HttpServer::start(None).await.unwrap();
        let target_port = http_server.port();

        // 2. Start SOCKS5 node
        let mut env = TestEnvironment::new();
        let socks_port = pick_port().await;

        let cfg: Config = serde_json::from_value(serde_json::json!({
            "inbounds": [{
                "tag": "socks-in",
                "port": socks_port,
                "listen": "127.0.0.1",
                "protocol": "socks"
            }],
            "outbounds": [{
                "tag": "direct",
                "protocol": "freedom"
            }]
        }))
        .unwrap();
        env.start_node(cfg).await.unwrap();

        // 3. Connect via SOCKS5 to the HTTP server
        let mut client = socks5_connect(socks_port, "127.0.0.1", target_port)
            .await
            .unwrap();

        let req = format!(
            "GET / HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
            target_port
        );
        client.write_all(req.as_bytes()).await.unwrap();

        let mut resp = Vec::new();
        client.read_to_end(&mut resp).await.unwrap();

        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut parsed = httparse::Response::new(&mut headers);
        if let Ok(httparse::Status::Complete(header_len)) = parsed.parse(&resp) {
            assert_eq!(parsed.code, Some(200));
            assert_eq!(&resp[header_len..], b"Home");
        } else {
            panic!("Failed to parse HTTP response over SOCKS5");
        }
    }

    #[tokio::test]
    async fn test_socks5_bridge_tcp() {
        // Echo server with XOR key
        let xor_key = b's';
        let tcp_server = TcpServer::start(None, Some(xor_processor(xor_key)), None)
            .await
            .unwrap();
        let target_port = tcp_server.port();

        let mut env = TestEnvironment::new();
        let server_socks_port = pick_port().await;
        let client_inbound_port = pick_port().await;

        // Node 1: SOCKS5 Server -> Freedom Outbound
        let server_cfg: Config = serde_json::from_value(serde_json::json!({
            "inbounds": [{
                "tag": "socks-server",
                "port": server_socks_port,
                "listen": "127.0.0.1",
                "protocol": "socks"
            }],
            "outbounds": [{
                "tag": "direct",
                "protocol": "freedom"
            }]
        }))
        .unwrap();
        env.start_node(server_cfg).await.unwrap();

        // Node 2: SOCKS5 Inbound -> SOCKS5 Outbound
        let client_cfg: Config = serde_json::from_value(serde_json::json!({
            "inbounds": [{
                "tag": "client-socks-in",
                "port": client_inbound_port,
                "listen": "127.0.0.1",
                "protocol": "socks"
            }],
            "outbounds": [{
                "tag": "proxy-socks-out",
                "protocol": "socks",
                "settings": {
                    "servers": [{
                        "address": "127.0.0.1",
                        "port": server_socks_port
                    }]
                }
            }]
        }))
        .unwrap();
        env.start_node(client_cfg).await.unwrap();

        // Connect to client inbound, which bridges through SOCKS outbound to SOCKS server to target
        let mut client = socks5_connect(client_inbound_port, "127.0.0.1", target_port)
            .await
            .unwrap();

        let payload = random_payload(2048);
        client.write_all(&payload).await.unwrap();

        let mut recv = vec![0u8; payload.len()];
        client.read_exact(&mut recv).await.unwrap();

        assert_eq!(recv, xor(&payload, xor_key));
    }
}
