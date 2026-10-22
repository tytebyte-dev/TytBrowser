#[cfg(test)]
mod tests {
    use crate::app::proxyman::{InboundHandlerConfig, KnownProtocols, SniffingConfig};
    use crate::app::stats::StatsChannel;

    #[tokio::test]
    async fn test_stats_channel_pub_sub() {
        let channel = StatsChannel::new(32, 5);
        let mut rx1 = channel.subscribe().unwrap();
        let mut rx2 = channel.subscribe().unwrap();

        assert_eq!(channel.subscriber_count(), 2);

        channel.publish(1024).unwrap();

        let val1 = rx1.recv().await.unwrap();
        let val2 = rx2.recv().await.unwrap();

        assert_eq!(val1, 1024);
        assert_eq!(val2, 1024);
    }

    #[test]
    fn test_proxyman_config_parsing() {
        assert_eq!(KnownProtocols::from_str("http"), Some(KnownProtocols::Http));
        assert_eq!(KnownProtocols::from_str("TLS"), Some(KnownProtocols::Tls));
        assert_eq!(
            KnownProtocols::from_str("fakedns"),
            Some(KnownProtocols::Fakedns)
        );
        assert_eq!(KnownProtocols::from_str("unknown"), None);

        let sniffing = SniffingConfig {
            enabled: true,
            dest_override: vec!["http".into(), "tls".into()],
            metadata_only: false,
            route_only: false,
        };

        let inbound = InboundHandlerConfig {
            tag: "in-1".into(),
            listen: "127.0.0.1".into(),
            port: 10808,
            protocol: "socks".into(),
            sniffing: Some(sniffing),
            stream_settings: None,
        };

        assert_eq!(inbound.tag, "in-1");
        assert!(inbound.sniffing.unwrap().enabled);
    }
}
