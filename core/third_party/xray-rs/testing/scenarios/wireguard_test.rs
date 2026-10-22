#[cfg(test)]
mod tests {
    use crate::common::net::{Address, Destination};
    use crate::common::protocol::SessionContext;
    use crate::features::outbound::OutboundHandler;
    use crate::proxy::wireguard::{Client, WireGuardConfig, WireGuardPeer};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UdpSocket;

    #[tokio::test]
    async fn test_wireguard_udp_relay() {
        let mock_server = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let server_addr = mock_server.local_addr().unwrap();

        tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            if let Ok((n, peer)) = mock_server.recv_from(&mut buf).await {
                let _ = mock_server.send_to(&buf[..n], peer).await;
            }
        });

        let config = WireGuardConfig {
            secret_key: "private-key-base64".into(),
            address: vec!["10.0.0.2/32".into()],
            peers: vec![WireGuardPeer {
                public_key: "peer-public-key-base64".into(),
                endpoint: server_addr.to_string(),
                keepalive: 25,
            }],
            mtu: Some(1420),
            reserved: Some(vec![0, 0, 0]),
        };

        let client = Client::new("wg-out", config);
        let session = SessionContext::new(
            "socks",
            Destination::new(Address::Domain("example.com".into()), 80),
        );
        let mut stream = client.connect(&session).await.unwrap();

        let msg = b"WireGuard Tunnel Test Message";
        stream.write_all(msg).await.unwrap();

        let mut recv = vec![0u8; msg.len()];
        stream.read_exact(&mut recv).await.unwrap();
        assert_eq!(&recv, msg);
    }
}
