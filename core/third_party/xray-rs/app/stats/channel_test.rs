// Module: app\stats\channel_test.rs
// 1:1 Rust unit test suite corresponding to Go app\stats\channel_test.go

#[cfg(test)]
mod tests {
    use super::super::channel::StatsChannel;
    use super::super::config_pb::ChannelConfig;

    #[tokio::test]
    async fn test_stats_channel_pub_sub() {
        let chan = StatsChannel::new(10, 10);
        let mut sub = chan.subscribe().unwrap();
        let _ = chan.publish(42);
        assert_eq!(sub.recv().await, Ok(42));
    }

    #[test]
    fn test_stats_channel_subscriber_limit_and_close() {
        let config = ChannelConfig::new(false, 2, 8);
        let chan = StatsChannel::from_config(&config);

        assert!(chan.is_running());
        let _sub1 = chan.subscribe().unwrap();
        let _sub2 = chan.subscribe().unwrap();
        // Limit of 2 reached
        assert!(chan.subscribe().is_err());

        chan.close();
        assert!(!chan.is_running());
        assert!(chan.publish(100).is_err());
        assert!(chan.subscribe().is_err());
    }
}
