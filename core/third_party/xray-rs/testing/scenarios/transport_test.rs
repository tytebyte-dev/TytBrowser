// Module: testing\scenarios\transport_test.rs
// Test transport internet stream settings and TCP dialing

#[cfg(test)]
mod tests {
    use crate::common::net::{Address, Destination};
    use crate::testing::servers::tcp::{Server as TcpServer, echo_processor};
    use crate::transport::internet::{StreamSettings, TcpDialer};

    #[tokio::test]
    async fn test_scenario_tcp_dialer_echo() {
        let tcp_server = TcpServer::start(None, Some(echo_processor()), None)
            .await
            .unwrap();
        let target_port = tcp_server.port();

        let dest = Destination::tcp(Address::Domain("127.0.0.1".into()), target_port);
        let stream = TcpDialer::dial(&dest).await;
        assert!(stream.is_ok());
    }

    #[test]
    fn test_scenario_stream_settings() {
        let mut settings = StreamSettings::default();
        settings.network = "ws".into();
        settings.security = "tls".into();
        assert_eq!(settings.network, "ws");
        assert_eq!(settings.security, "tls");
    }
}
