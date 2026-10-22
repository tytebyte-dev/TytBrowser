#[cfg(test)]
mod tests {
    use crate::app::proxyman::outbound::DefaultOutboundManager;
    use crate::common::errors::Result;
    use crate::common::net::{Address, BoxStream, Destination};
    use crate::common::protocol::SessionContext;
    use crate::core::Instance;
    use crate::features::outbound::{OutboundHandler, OutboundManager};
    use crate::infra::conf::Config;
    use crate::transport::internet::tagged::TaggedDialer;
    use async_trait::async_trait;
    use std::sync::Arc;
    use tokio::io::duplex;

    struct DummyOutbound {
        tag: String,
    }

    #[async_trait]
    impl OutboundHandler for DummyOutbound {
        fn tag(&self) -> &str {
            &self.tag
        }

        async fn connect(&self, _session: &SessionContext) -> Result<BoxStream> {
            let (client, _server) = duplex(1024);
            Ok(Box::pin(client))
        }
    }

    #[tokio::test]
    async fn test_tagged_dialer_connect() {
        let manager = Arc::new(DefaultOutboundManager::new());
        let handler = Arc::new(DummyOutbound {
            tag: "direct-out".into(),
        });
        manager.add_handler(handler).await.unwrap();

        let dialer = TaggedDialer::new(manager);
        let dest = Destination::tcp(Address::Domain("google.com".into()), 443);

        let stream = dialer.dial("direct-out", dest.clone()).await;
        assert!(stream.is_ok());

        let err_stream = dialer.dial("non-existent-tag", dest).await;
        assert!(err_stream.is_err());
    }

    #[test]
    fn test_instance_from_minimal_json_config() {
        let json = r#"{
            "inbounds": [{
                "port": 10808,
                "protocol": "socks",
                "settings": {
                    "auth": "noauth"
                }
            }],
            "outbounds": [{
                "protocol": "freedom"
            }]
        }"#;

        let config: Config = serde_json::from_str(json).unwrap();
        let instance = Instance::from_config(config);
        assert!(instance.is_ok());
    }
}
