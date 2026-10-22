#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
    use std::sync::Arc;
    use std::time::Duration;

    use crate::common::buf::{Buffer, MultiBuffer, TimeoutReader, Writer};
    use crate::common::ctx::Context;
    use crate::common::errors::{Error, Result};
    use crate::common::net::{BoxStream, Destination, Network};
    use crate::common::protocol::SessionContext;
    use crate::common::singbridge::*;
    use crate::features::feature::Feature;
    use crate::features::outbound::OutboundHandler;
    use crate::features::routing::Dispatcher as RoutingDispatcher;
    use crate::transport::Link;
    use crate::transport::internet::dialer::Dialer as InternetDialer;
    use crate::transport::pipe::{PipeOption, new_pipe};

    #[test]
    fn test_destination_and_socksaddr_conversion() {
        assert_eq!(to_network("tcp"), Network::Tcp);
        assert_eq!(to_network("TCP"), Network::Tcp);
        assert_eq!(to_network("udp"), Network::Udp);
        assert_eq!(to_network("UDP"), Network::Udp);
        assert_eq!(to_network("unknown"), Network::Tcp);

        // IPv4 Socksaddr roundtrip
        let ipv4 = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let sa_ip = Socksaddr::new_ip(ipv4, 8080);
        assert!(sa_ip.is_ip());
        assert!(!sa_ip.is_fqdn());
        assert_eq!(sa_ip.to_socket_addr(), Some(SocketAddr::new(ipv4, 8080)));

        let dest_ip = to_destination(&sa_ip, Network::Tcp);
        assert_eq!(dest_ip.network, Network::Tcp);
        assert_eq!(dest_ip.port, 8080);
        assert_eq!(dest_ip.address.to_ip(), Some(ipv4));

        let sa_back = to_socksaddr(&dest_ip);
        assert_eq!(sa_back, sa_ip);

        // IPv6 Socksaddr roundtrip
        let ipv6 = IpAddr::V6(Ipv6Addr::LOCALHOST);
        let sa_v6 = Socksaddr::new_ip(ipv6, 9000);
        let dest_v6 = to_destination(&sa_v6, Network::Udp);
        assert_eq!(dest_v6.network, Network::Udp);
        assert_eq!(dest_v6.port, 9000);
        assert_eq!(to_socksaddr(&dest_v6), sa_v6);

        // FQDN Socksaddr roundtrip
        let sa_fqdn = Socksaddr::new_fqdn("example.com", 443);
        assert!(sa_fqdn.is_fqdn());
        assert!(!sa_fqdn.is_ip());
        assert_eq!(sa_fqdn.fqdn_str(), "example.com");

        let dest_fqdn = to_destination(&sa_fqdn, Network::Tcp);
        assert_eq!(dest_fqdn.address.domain_name(), Some("example.com"));
        assert_eq!(dest_fqdn.port, 443);

        let sa_fqdn_back = to_socksaddr(&dest_fqdn);
        assert_eq!(sa_fqdn_back, sa_fqdn);
    }

    #[test]
    fn test_error_classification_and_filtering() {
        let broken_pipe = Error::Io(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "pipe broken",
        ));
        let conn_reset = Error::Io(std::io::Error::new(
            std::io::ErrorKind::ConnectionReset,
            "reset",
        ));
        let other_err = Error::Other("generic error".into());

        assert!(is_closed_or_canceled(&broken_pipe));
        assert!(is_closed_or_canceled(&conn_reset));
        assert!(!is_closed_or_canceled(&other_err));

        assert!(return_error(Some(broken_pipe)).is_none());
        assert!(return_error(Some(conn_reset)).is_none());
        assert!(return_error(Some(other_err)).is_some());
        assert!(return_error(None).is_none());

        let ok_res: Result<i32> = Ok(42);
        assert_eq!(return_result(ok_res).unwrap(), Some(42));

        let err_cancel: Result<i32> = Err(Error::Other("context canceled".into()));
        assert_eq!(return_result(err_cancel).unwrap(), None);

        let custom_err = wrap_sing_error("failed to initialize");
        assert!(custom_err.to_string().contains("sing-box bridge error"));
    }

    #[test]
    fn test_xray_logger_levels() {
        let logger = XrayLogger::new();
        let ctx = Context::new();

        logger.trace("test trace");
        logger.debug("test debug");
        logger.info("test info");
        logger.warn("test warn");
        logger.error("test error");
        logger.fatal("test fatal");
        logger.panic("test panic");

        logger.trace_context(&ctx, "test trace ctx");
        logger.debug_context(&ctx, "test debug ctx");
        logger.info_context(&ctx, "test info ctx");
        logger.warn_context(&ctx, "test warn ctx");
        logger.error_context(&ctx, "test error ctx");
        logger.fatal_context(&ctx, "test fatal ctx");
        logger.panic_context(&ctx, "test panic ctx");
    }

    #[tokio::test]
    async fn test_conn_reader_and_writer() {
        let (client_stream, server_stream) = tokio::io::duplex(4096);
        let mut client_conn = Conn::new(Box::pin(client_stream));
        let mut server_conn = Conn::new(Box::pin(server_stream));

        let payload = b"hello singbridge connection";
        let mut mb = MultiBuffer::new();
        let mut b = Buffer::new();
        let _ = b.write(payload);
        mb.push(b);

        client_conn.write_multi_buffer(mb).await.unwrap();

        let read_mb = server_conn
            .read_multi_buffer_timeout(Duration::from_secs(2))
            .await
            .unwrap();
        assert_eq!(read_mb.to_vec(), payload);
    }

    #[tokio::test]
    async fn test_pipe_conn_wrapper_roundtrip() {
        let (reader, writer) = new_pipe(PipeOption::new());
        let mut wrapper = PipeConnWrapper::new(reader, writer);

        let data = b"piped data payload";
        wrapper.write(data).await.unwrap();

        let read_bytes = wrapper.read().await.unwrap();
        assert_eq!(read_bytes, data);

        wrapper.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_packet_conn_wrapper() {
        let (reader, writer) = new_pipe(PipeOption::new());
        let dest = Destination::udp(
            crate::common::net::Address::from(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))),
            53,
        );
        let link = Link::new(reader, writer);
        let mut wrapper = PacketConnWrapper::from_link(&link, dest.clone());

        // Write a packet
        let mut b = Buffer::new();
        let _ = b.write(b"dns query packet");
        let target_sa = to_socksaddr(&dest);
        wrapper.write_packet(b, target_sa.clone()).await.unwrap();

        // Read the packet back
        let (pkt, sa) = wrapper.read_packet().await.unwrap();
        assert_eq!(pkt.as_slice(), b"dns query packet");
        assert_eq!(sa, target_sa);

        wrapper.close().await.unwrap();
    }

    struct MockInternetDialer;

    #[async_trait]
    impl InternetDialer for MockInternetDialer {
        async fn dial(&self, _dest: &Destination) -> Result<BoxStream> {
            let (c, _s) = tokio::io::duplex(1024);
            Ok(Box::pin(c))
        }
    }

    struct MockOutboundHandler;

    #[async_trait]
    impl OutboundHandler for MockOutboundHandler {
        fn tag(&self) -> &str {
            "mock-out"
        }

        async fn connect(&self, _session: &SessionContext) -> Result<BoxStream> {
            let (c, _s) = tokio::io::duplex(1024);
            Ok(Box::pin(c))
        }
    }

    #[tokio::test]
    async fn test_xray_dialer_and_outbound_dialer() {
        let ctx = Context::new();
        let target_sa = Socksaddr::new_ip(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), 80);

        // Test XrayDialer
        let dialer = XrayDialer::new(Arc::new(MockInternetDialer));
        let stream = dialer.dial_context(&ctx, "tcp", target_sa.clone()).await;
        assert!(stream.is_ok());

        assert!(dialer.listen_packet(&ctx, target_sa.clone()).await.is_err());

        // Test XrayOutboundDialer
        let ob_dialer = XrayOutboundDialer::new(Arc::new(MockOutboundHandler), None);
        let ob_stream = ob_dialer.dial_context(&ctx, "tcp", target_sa).await;
        assert!(ob_stream.is_ok());
    }

    struct MockRoutingDispatcher;

    impl Feature for MockRoutingDispatcher {
        fn feature_type(&self) -> &'static str {
            crate::features::feature::TYPE_DISPATCHER
        }
    }

    #[async_trait]
    impl RoutingDispatcher for MockRoutingDispatcher {
        async fn dispatch(&self, _session: SessionContext, _stream: BoxStream) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_dispatcher_handlers() {
        let ctx = Context::new();
        let upstream = Arc::new(MockRoutingDispatcher);
        let dispatcher = Dispatcher::new(upstream);

        let src = Socksaddr::new_ip(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 12345);
        let dst = Socksaddr::new_ip(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), 53);
        let meta = Metadata::new(src, dst);

        let (c1, _s1) = tokio::io::duplex(1024);
        let (c2, _s2) = tokio::io::duplex(1024);

        let res_tcp = dispatcher
            .new_connection(&ctx, Box::pin(c1), meta.clone())
            .await;
        assert!(res_tcp.is_ok());

        let res_udp = dispatcher
            .new_packet_connection(&ctx, Box::pin(c2), meta)
            .await;
        assert!(res_udp.is_ok());

        dispatcher.new_error(&ctx, &Error::Other("test error log".into()));
    }
}
