#[cfg(test)]
mod tests {
    use crate::common::errors::{Error, MultiError};
    use crate::common::log::{AccessLogMessage, AccessStatus};
    use crate::common::net::{Address, Destination};
    use crate::common::signal::PubSubService;
    use std::net::SocketAddr;

    #[tokio::test]
    async fn test_pubsub_service_broadcast_and_cleanup() {
        let pubsub = PubSubService::new(16);
        let mut sub1 = pubsub.subscribe("routing-events");
        let mut sub2 = pubsub.subscribe("routing-events");

        pubsub
            .publish("routing-events", "node-switched-to-direct")
            .unwrap();

        assert_eq!(sub1.recv().await.unwrap(), "node-switched-to-direct");
        assert_eq!(sub2.recv().await.unwrap(), "node-switched-to-direct");

        drop(sub1);
        drop(sub2);
        pubsub.cleanup();
    }

    #[test]
    fn test_multi_error_collection_and_formatting() {
        let mut multi = MultiError::new();
        assert!(!multi.has_errors());
        assert_eq!(multi.len(), 0);

        multi.add(Error::Config("Invalid port 99999".into()));
        multi.add(Error::Timeout);
        assert!(multi.has_errors());
        assert_eq!(multi.len(), 2);

        let err_str = multi.to_string();
        assert!(err_str.contains("Invalid port 99999"));
        assert!(err_str.contains("Timeout"));
    }

    #[test]
    fn test_access_log_message_formatting() {
        let msg = AccessLogMessage {
            from: Some("127.0.0.1:12345".parse::<SocketAddr>().unwrap()),
            to: Destination::tcp(Address::Domain("google.com".into()), 443),
            status: AccessStatus::Accepted,
            reason: "match-domain-rule".into(),
        };

        let formatted = msg.format();
        assert_eq!(
            formatted,
            "127.0.0.1:12345 accepted google.com:443 [match-domain-rule]"
        );
    }
}
