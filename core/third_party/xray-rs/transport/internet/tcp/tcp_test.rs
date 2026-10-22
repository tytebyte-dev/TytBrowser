// Module: transport\internet\tcp\tcp_test.rs
// 1:1 Rust unit test suite corresponding to Go transport\internet\tcp

#[cfg(test)]
mod tests {
    use super::super::config::TcpConfig;
    use super::super::dialer::TcpDialer;
    use super::super::hub::TcpHub;
    use super::super::tcp::PROTOCOL_NAME;
    use crate::common::net::{Address, Destination, Network};
    use crate::transport::internet::tcp_hub::listen_tcp;
    use std::net::{Ipv4Addr, SocketAddr};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[test]
    fn test_tcp_constants_and_config() {
        assert_eq!(PROTOCOL_NAME, "tcp");
        let cfg = TcpConfig::new();
        assert!(!cfg.accept_proxy_protocol);
        assert!(cfg.header_type.is_none());
    }

    #[tokio::test]
    async fn test_tcp_listen_and_reject_domain() {
        // Domain rejected
        let domain_addr = Address::domain("example.com");
        let res = listen_tcp(&domain_addr, 8080).await;
        assert!(res.is_err());

        // Localhost allowed and bound to ephemeral port
        let local_addr = Address::domain("localhost");
        let listener = listen_tcp(&local_addr, 0).await.expect("localhost bind");
        assert!(listener.local_addr().is_ok());
    }

    #[tokio::test]
    async fn test_tcp_hub_and_dialer_duplex() {
        let bind_addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let hub = TcpHub::listen(bind_addr).await.expect("tcp hub listen");
        let local_addr = hub.local_addr();

        let server_task = tokio::spawn(async move {
            let (mut stream, _remote) = hub.accept().await.expect("accept");
            let mut buf = [0u8; 16];
            let n = stream.read(&mut buf).await.expect("read");
            assert_eq!(&buf[..n], b"Hello TCP");

            stream.write_all(b"Reply TCP").await.expect("write");
            stream.flush().await.expect("flush");
        });

        let client_task = tokio::spawn(async move {
            let dest = Destination {
                network: Network::Tcp,
                address: Address::Ipv4(Ipv4Addr::LOCALHOST),
                port: local_addr.port(),
            };

            let mut stream = TcpDialer::dial(&dest).await.expect("dial");
            stream.write_all(b"Hello TCP").await.expect("client write");
            stream.flush().await.expect("client flush");

            let mut buf = [0u8; 16];
            let n = stream.read(&mut buf).await.expect("client read");
            assert_eq!(&buf[..n], b"Reply TCP");
        });

        let (s_res, c_res) = tokio::join!(server_task, client_task);
        s_res.expect("server join");
        c_res.expect("client join");
    }
}
