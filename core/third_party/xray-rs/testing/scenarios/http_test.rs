// Module: testing\scenarios\http_test.rs
// Real end-to-end integration tests for HTTP inbound/outbound proxy

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use crate::infra::conf::Config;
    use crate::testing::scenarios::common::{
        TestEnvironment, http_connect_tunnel, http_get_proxy, pick_port, random_payload, xor,
    };
    use crate::testing::servers::http::Server as HttpServer;
    use crate::testing::servers::tcp::{Server as TcpServer, xor_processor};

    #[tokio::test]
    async fn test_http_conformance() {
        // 1. Start target HTTP server
        let http_server = HttpServer::start(None).await.unwrap();
        let http_port = http_server.port();

        // 2. Start Xray node with HTTP inbound and Freedom outbound
        let mut env = TestEnvironment::new();
        let proxy_port = pick_port().await;

        let node_cfg: Config = serde_json::from_value(serde_json::json!({
            "inbounds": [{
                "tag": "http-in",
                "port": proxy_port,
                "listen": "127.0.0.1",
                "protocol": "http"
            }],
            "outbounds": [{
                "tag": "direct",
                "protocol": "freedom"
            }]
        }))
        .unwrap();
        env.start_node(node_cfg).await.unwrap();

        // 3. Issue HTTP GET through proxy
        let target_url = format!("http://127.0.0.1:{}/", http_port);
        let (status, body) = http_get_proxy(proxy_port, &target_url).await.unwrap();

        assert_eq!(status, 200);
        assert_eq!(body, b"Home");
    }

    #[tokio::test]
    async fn test_http_connect_method() {
        // 1. Start TCP server with XOR processing
        let xor_key = b'k';
        let tcp_server = TcpServer::start(None, Some(xor_processor(xor_key)), None)
            .await
            .unwrap();
        let target_port = tcp_server.port();

        // 2. Start Xray node with HTTP inbound and Freedom outbound
        let mut env = TestEnvironment::new();
        let proxy_port = pick_port().await;

        let node_cfg: Config = serde_json::from_value(serde_json::json!({
            "inbounds": [{
                "tag": "http-in",
                "port": proxy_port,
                "listen": "127.0.0.1",
                "protocol": "http"
            }],
            "outbounds": [{
                "tag": "direct",
                "protocol": "freedom"
            }]
        }))
        .unwrap();
        env.start_node(node_cfg).await.unwrap();

        // 3. Establish HTTP CONNECT tunnel through proxy
        let mut tunnel = http_connect_tunnel(proxy_port, "127.0.0.1", target_port)
            .await
            .unwrap();

        // 4. Send binary payload through the tunnel
        let payload = random_payload(4096);
        tunnel.write_all(&payload).await.unwrap();

        // 5. Read back response and verify XOR
        let mut resp = vec![0u8; payload.len()];
        tunnel.read_exact(&mut resp).await.unwrap();

        assert_eq!(resp, xor(&payload, xor_key));
    }

    #[tokio::test]
    async fn test_http_post() {
        // 1. Start HTTP server with /testpost handler that echoes XORed body
        let mut handlers = HashMap::new();
        handlers.insert(
            "/testpost".to_string(),
            Arc::new(|method: &str, _path: &str, body: &[u8]| {
                if method.eq_ignore_ascii_case("POST") {
                    (200u16, xor(body, b'p'))
                } else {
                    (405u16, b"Method Not Allowed".to_vec())
                }
            }) as _,
        );

        let http_server = HttpServer::start_with_handlers(None, handlers)
            .await
            .unwrap();
        let http_port = http_server.port();

        // 2. Start Xray node
        let mut env = TestEnvironment::new();
        let proxy_port = pick_port().await;

        let node_cfg: Config = serde_json::from_value(serde_json::json!({
            "inbounds": [{
                "tag": "http-in",
                "port": proxy_port,
                "listen": "127.0.0.1",
                "protocol": "http"
            }],
            "outbounds": [{
                "tag": "direct",
                "protocol": "freedom"
            }]
        }))
        .unwrap();
        env.start_node(node_cfg).await.unwrap();

        // 3. Connect to proxy and send raw HTTP POST request
        let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", proxy_port))
            .await
            .unwrap();
        let post_body = b"Xray-Rust HTTP POST Test Payload";
        let req = format!(
            "POST http://127.0.0.1:{}/testpost HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            http_port,
            http_port,
            post_body.len()
        );

        stream.write_all(req.as_bytes()).await.unwrap();
        stream.write_all(post_body).await.unwrap();
        stream.flush().await.unwrap();

        let mut resp_buf = Vec::new();
        stream.read_to_end(&mut resp_buf).await.unwrap();

        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut parsed = httparse::Response::new(&mut headers);
        if let Ok(httparse::Status::Complete(header_len)) = parsed.parse(&resp_buf) {
            assert_eq!(parsed.code, Some(200));
            let body = &resp_buf[header_len..];
            assert_eq!(body, xor(post_body, b'p'));
        } else {
            panic!("Failed to parse HTTP POST response");
        }
    }
}
