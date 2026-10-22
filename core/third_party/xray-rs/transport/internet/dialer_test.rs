// Module: transport\internet\dialer_test.rs
// 1:1 Rust unit test suite corresponding to Go transport\internet\dialer_test.go

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    use crate::common::net::{Address, Destination};
    use crate::transport::internet::config::DomainStrategy;
    use crate::transport::internet::dialer::{
        DefaultDialer, Dialer, dial_system, dial_transport, register_transport_dialer,
    };
    use crate::transport::internet::memory_settings::MemoryStreamConfig;

    #[tokio::test]
    async fn test_default_dialer_loopback() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut buf = [0u8; 4];
                let _ = stream.read_exact(&mut buf).await;
                let _ = stream.write_all(b"pong").await;
            }
        });

        let dialer = DefaultDialer;
        let dest = Destination::tcp(Address::ip("127.0.0.1".parse().unwrap()), port);
        let mut conn = dialer.dial(&dest).await.unwrap();

        conn.write_all(b"ping").await.unwrap();
        let mut buf = [0u8; 4];
        conn.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"pong");
    }

    #[tokio::test]
    async fn test_dial_system_direct() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let dest = Destination::tcp(Address::ip("127.0.0.1".parse().unwrap()), port);
        let conn = dial_system(&dest, None).await;
        assert!(conn.is_ok());
    }

    #[tokio::test]
    async fn test_register_and_dial_transport() {
        let invoked = Arc::new(AtomicBool::new(false));
        let invoked_clone = Arc::clone(&invoked);

        let custom_proto = "custom-mock-proto";
        let _ = register_transport_dialer(
            custom_proto,
            Arc::new(move |_dest, _settings| {
                let called = Arc::clone(&invoked_clone);
                Box::pin(async move {
                    called.store(true, Ordering::SeqCst);
                    let (tx, _rx) = tokio::io::duplex(64);
                    Ok(Box::pin(tx) as crate::common::net::BoxStream)
                })
            }),
        );

        let dest = Destination::tcp(Address::ip("127.0.0.1".parse().unwrap()), 80);
        let stream_settings = MemoryStreamConfig {
            protocol_name: custom_proto.to_string(),
            socket_settings: None,
        };

        let stream = dial_transport(&dest, Some(&stream_settings)).await;
        assert!(stream.is_ok());
        assert!(invoked.load(Ordering::SeqCst));
    }

    #[test]
    fn test_domain_strategy_evaluations() {
        let as_is = DomainStrategy::AsIs;
        assert!(!as_is.has_strategy());
        assert!(!as_is.force_ip());

        let force_ip = DomainStrategy::ForceIp;
        assert!(force_ip.has_strategy());
        assert!(force_ip.force_ip());

        let use_ip4 = DomainStrategy::UseIp4;
        assert!(use_ip4.prefer_ip4());
        assert!(!use_ip4.has_fallback());

        let use_ip46 = DomainStrategy::UseIp46;
        assert!(use_ip46.has_fallback());
        assert!(use_ip46.fallback_ip6());
    }
}
