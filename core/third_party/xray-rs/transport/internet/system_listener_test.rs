// Module: transport\internet\system_listener_test.rs
// 1:1 Rust unit test suite corresponding to Go transport\internet\system_listener_test.go

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use crate::common::net::{Address, Destination};
    use crate::transport::internet::system_dialer::{PacketConnWrapper, SystemDialer};
    use crate::transport::internet::system_listener::SystemListener;

    #[tokio::test]
    async fn test_system_listener_bind() {
        let listener = SystemListener::bind("127.0.0.1:0".parse().unwrap())
            .await
            .unwrap();
        let addr = listener.local_addr().unwrap();
        assert_ne!(addr.port(), 0);
    }

    #[tokio::test]
    async fn test_system_listener_listen_packet() {
        let socket = SystemListener::listen_packet("127.0.0.1:0".parse().unwrap(), None)
            .await
            .unwrap();
        let addr = socket.local_addr().unwrap();
        assert_ne!(addr.port(), 0);
    }

    #[tokio::test]
    async fn test_system_listener_controller() {
        let listener = SystemListener::bind("127.0.0.1:0".parse().unwrap())
            .await
            .unwrap();
        let addr = listener.local_addr().unwrap();

        let called = Arc::new(AtomicBool::new(false));
        let called_clone = Arc::clone(&called);

        listener
            .add_controller(Arc::new(move |_net, _addr| {
                called_clone.store(true, Ordering::SeqCst);
                Ok(())
            }))
            .await;

        tokio::spawn(async move {
            let _ = listener.accept().await;
        });

        let dest = Destination::tcp(Address::ip(addr.ip()), addr.port());
        let mut stream = SystemDialer::dial_tcp(&dest).await.unwrap();
        stream.write_all(b"ping").await.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        assert!(called.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_system_dialer_duplex() {
        let listener = SystemListener::bind("127.0.0.1:0".parse().unwrap())
            .await
            .unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut buf = [0u8; 4];
                let _ = stream.read_exact(&mut buf).await;
                let _ = stream.write_all(b"pong").await;
            }
        });

        let dest = Destination::tcp(Address::ip(addr.ip()), addr.port());
        let mut client = SystemDialer::dial_tcp(&dest).await.unwrap();
        client.write_all(b"ping").await.unwrap();

        let mut buf = [0u8; 4];
        client.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"pong");
    }

    #[tokio::test]
    async fn test_packet_conn_wrapper() {
        let server_socket = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let server_addr = server_socket.local_addr().unwrap();

        let client_socket = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let mut wrapper = PacketConnWrapper::new(client_socket, server_addr);

        assert_eq!(wrapper.remote_addr(), server_addr);

        wrapper.write_all(b"udp-ping").await.unwrap();

        let mut sbuf = [0u8; 8];
        let (n, from) = server_socket.recv_from(&mut sbuf).await.unwrap();
        assert_eq!(&sbuf[..n], b"udp-ping");

        server_socket.send_to(b"udp-pong", from).await.unwrap();

        let mut cbuf = [0u8; 8];
        wrapper.read_exact(&mut cbuf).await.unwrap();
        assert_eq!(&cbuf, b"udp-pong");
    }
}
