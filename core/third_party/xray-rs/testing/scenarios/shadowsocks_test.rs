#[cfg(test)]
mod tests {
    use crate::common::net::{Address, Destination};
    use crate::infra::conf::Config;
    use crate::proxy::shadowsocks::protocol::{ShadowsocksUdpPacket, derive_subkey};
    use crate::testing::scenarios::common::{TestEnvironment, pick_port};
    use crate::testing::servers::tcp::{Server as TcpServer, echo_processor};
    use std::io::Cursor;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[test]
    fn test_shadowsocks_hkdf_subkey_derivation() {
        let master_key = b"password_master_key_12345678901";
        let salt = b"unique_salt_1234";
        let mut subkey1 = [0u8; 32];
        let mut subkey2 = [0u8; 32];

        derive_subkey(master_key, salt, &mut subkey1).unwrap();
        derive_subkey(master_key, salt, &mut subkey2).unwrap();

        assert_eq!(subkey1, subkey2);
        assert_ne!(subkey1, [0u8; 32]);
    }

    #[tokio::test]
    async fn test_shadowsocks_udp_packet_roundtrip() {
        let target = Destination::udp(Address::Domain("dns.google".into()), 53);
        let payload = b"Shadowsocks UDP query payload";

        let packet = ShadowsocksUdpPacket::new(target.clone(), payload.to_vec());

        let mut buf = Vec::new();
        packet.encode(&mut buf).await.unwrap();

        let mut cursor = Cursor::new(buf);
        let decoded = ShadowsocksUdpPacket::decode(&mut cursor).await.unwrap();

        assert_eq!(decoded.destination.address, target.address);
        assert_eq!(decoded.destination.port, target.port);
        assert_eq!(decoded.payload, payload);
    }

    #[tokio::test]
    async fn test_shadowsocks_tcp_relay() {
        let tcp_server = TcpServer::start(None, Some(echo_processor()), None)
            .await
            .unwrap();
        let target_port = tcp_server.port();

        let mut env = TestEnvironment::new();
        let password = "shadowsocks_secret_key";
        let method = "aes-256-gcm";

        let server_port = pick_port().await;
        let server_cfg: Config = serde_json::from_value(serde_json::json!({
            "inbounds": [{
                "tag": "ss-in",
                "port": server_port,
                "listen": "127.0.0.1",
                "protocol": "shadowsocks",
                "settings": {
                    "servers": [{
                        "password": password,
                        "method": method
                    }]
                }
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
                "protocol": "shadowsocks",
                "settings": {
                    "servers": [{
                        "address": "127.0.0.1",
                        "port": server_port,
                        "password": password,
                        "method": method
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

        let mut socks_resp = [0u8; 10];
        client.read_exact(&mut socks_resp).await.unwrap();
        assert_eq!(socks_resp[1], 0x00);

        let msg = b"ping shadowsocks";
        client.write_all(msg).await.unwrap();

        let mut echo = vec![0u8; msg.len()];
        client.read_exact(&mut echo).await.unwrap();
        assert_eq!(&echo, msg);
    }
}
