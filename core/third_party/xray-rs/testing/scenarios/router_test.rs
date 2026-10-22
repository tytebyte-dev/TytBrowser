#[cfg(test)]
mod tests {
    use crate::testing::scenarios::common::{TestEnvironment, pick_port};
    use crate::testing::servers::tcp::{Server as TcpServer, echo_processor, xor_processor};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_router_rule_dispatching() {
        let direct_server = TcpServer::start(None, Some(echo_processor()), None)
            .await
            .unwrap();
        let direct_port = direct_server.port();

        let proxy_target_server = TcpServer::start(None, Some(xor_processor(b'k')), None)
            .await
            .unwrap();
        let proxy_target_port = proxy_target_server.port();

        let mut env = TestEnvironment::new();
        let user_id = Uuid::new_v4();

        let vless_port = pick_port().await;
        env.start_node(
            serde_json::from_value(serde_json::json!({
                "inbounds": [{
                    "tag": "vless-in",
                    "port": vless_port,
                    "listen": "127.0.0.1",
                    "protocol": "vless",
                    "settings": { "clients": [{ "id": user_id.to_string() }] }
                }],
                "outbounds": [{ "tag": "direct", "protocol": "freedom" }]
            }))
            .unwrap(),
        )
        .await
        .unwrap();

        let socks_port = pick_port().await;
        env.start_node(
            serde_json::from_value(serde_json::json!({
                "inbounds": [{
                    "tag": "socks-in",
                    "port": socks_port,
                    "listen": "127.0.0.1",
                    "protocol": "socks"
                }],
                "outbounds": [
                    { "tag": "direct", "protocol": "freedom" },
                    {
                        "tag": "proxy-vless",
                        "protocol": "vless",
                        "settings": {
                            "vnext": [{
                                "address": "127.0.0.1",
                                "port": vless_port,
                                "users": [{ "id": user_id.to_string() }]
                            }]
                        }
                    },
                    { "tag": "blocked", "protocol": "blackhole" }
                ],
                "routing": {
                    "rules": [
                        {
                            "type": "field",
                            "outboundTag": "blocked",
                            "domain": ["keyword:block"]
                        },
                        {
                            "type": "field",
                            "outboundTag": "proxy-vless",
                            "domain": ["keyword:proxy"]
                        }
                    ]
                }
            }))
            .unwrap(),
        )
        .await
        .unwrap();

        let mut c1 = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", socks_port))
            .await
            .unwrap();
        c1.write_all(&[0x05, 0x01, 0x00]).await.unwrap();
        let mut resp = [0u8; 2];
        c1.read_exact(&mut resp).await.unwrap();

        let domain = "my-proxy.com";
        let mut req = vec![0x05, 0x01, 0x00, 0x03, domain.len() as u8];
        req.extend_from_slice(domain.as_bytes());
        req.extend_from_slice(&proxy_target_port.to_be_bytes());
        c1.write_all(&req).await.unwrap();

        let mut connect_resp = [0u8; 10];
        c1.read_exact(&mut connect_resp).await.unwrap();
        assert_eq!(connect_resp[1], 0x00);

        let mut c2 = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", socks_port))
            .await
            .unwrap();
        c2.write_all(&[0x05, 0x01, 0x00]).await.unwrap();
        c2.read_exact(&mut resp).await.unwrap();

        let block_domain = "ad-block.com";
        let mut block_req = vec![0x05, 0x01, 0x00, 0x03, block_domain.len() as u8];
        block_req.extend_from_slice(block_domain.as_bytes());
        block_req.extend_from_slice(&direct_port.to_be_bytes());
        c2.write_all(&block_req).await.unwrap();
        c2.read_exact(&mut connect_resp).await.unwrap();

        c2.write_all(b"Hello").await.unwrap();
        let mut buf = [0u8; 10];
        let n = c2.read(&mut buf).await.unwrap_or(0);
        assert_eq!(n, 0, "Blocked route must terminate connection");
    }

    #[tokio::test]
    async fn test_udp_rule_does_not_block_tcp() {
        let echo_server = TcpServer::start(None, Some(echo_processor()), None)
            .await
            .unwrap();
        let echo_port = echo_server.port();

        let mut env = TestEnvironment::new();
        let socks_port = pick_port().await;

        let cfg: serde_json::Value = serde_json::json!({
            "inbounds": [{
                "tag": "socks-in",
                "port": socks_port,
                "listen": "127.0.0.1",
                "protocol": "socks"
            }],
            "outbounds": [
                { "tag": "direct", "protocol": "freedom" },
                { "tag": "block", "protocol": "blackhole" }
            ],
            "routing": {
                "rules": [
                    {
                        "port": format!("{}", echo_port),
                        "network": "udp",
                        "outboundTag": "block"
                    }
                ]
            }
        });
        env.start_node(serde_json::from_value(cfg).unwrap())
            .await
            .unwrap();

        // Connect via TCP: must NOT be blocked because rule only applies to UDP!
        let mut client = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", socks_port))
            .await
            .unwrap();
        client.write_all(&[0x05, 0x01, 0x00]).await.unwrap();
        let mut auth_resp = [0u8; 2];
        client.read_exact(&mut auth_resp).await.unwrap();

        let mut req = vec![0x05, 0x01, 0x00, 0x01, 127, 0, 0, 1];
        req.extend_from_slice(&echo_port.to_be_bytes());
        client.write_all(&req).await.unwrap();

        let mut connect_resp = [0u8; 10];
        client.read_exact(&mut connect_resp).await.unwrap();
        assert_eq!(connect_resp[1], 0x00);

        let msg = b"TCP Connection Must Not Match UDP Rule";
        client.write_all(msg).await.unwrap();
        let mut recv = vec![0u8; msg.len()];
        client.read_exact(&mut recv).await.unwrap();
        assert_eq!(&recv, msg);
    }
}
