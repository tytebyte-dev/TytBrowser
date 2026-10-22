#[cfg(test)]
mod tests {
    use crate::common::net::{Address, Destination, Network};
    use crate::features::inbound::InboundHandler;
    use crate::proxy::socks::Server;
    use crate::proxy::socks::protocol::SocksProtocol;
    use std::net::SocketAddr;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn test_socks5_client_and_server_handshake() {
        let server = Server::new("socks-in");
        let (client_stream, server_stream) = tokio::io::duplex(256);

        let target = Destination {
            network: Network::Tcp,
            address: Address::Domain("www.example.com".to_string()),
            port: 443,
        };

        let server_task = tokio::spawn(async move {
            let fake_remote: SocketAddr = "127.0.0.1:54321".parse().unwrap();
            let res = server
                .handle_connection(Box::pin(server_stream), fake_remote)
                .await
                .unwrap();
            assert_eq!(res.session.destination.port, 443);
            assert_eq!(
                res.session.destination.address,
                Address::Domain("www.example.com".to_string())
            );

            let mut stream = res.stream;
            let mut buf = [0u8; 4];
            stream.read_exact(&mut buf).await.unwrap();
            assert_eq!(&buf, b"ping");
            stream.write_all(b"pong").await.unwrap();
        });

        let mut client = client_stream;
        SocksProtocol::client_handshake(&mut client, &target)
            .await
            .unwrap();

        client.write_all(b"ping").await.unwrap();
        let mut resp = [0u8; 4];
        client.read_exact(&mut resp).await.unwrap();
        assert_eq!(&resp, b"pong");

        server_task.await.unwrap();
    }
}
