#[cfg(test)]
mod tests {
    use crate::app::proxyman::inbound::DefaultInboundManager;
    use crate::app::proxyman::outbound::DefaultOutboundManager;
    use crate::common::errors::Result;
    use crate::common::net::BoxStream;
    use crate::common::protocol::SessionContext;
    use crate::features::inbound::{InboundHandler, InboundManager, InboundResult};
    use crate::features::outbound::{HandlerSelector, OutboundHandler, OutboundManager};
    use async_trait::async_trait;
    use std::net::SocketAddr;
    use std::sync::Arc;
    use tokio::io::duplex;

    struct MockInboundHandler {
        tag: String,
    }

    #[async_trait]
    impl InboundHandler for MockInboundHandler {
        fn tag(&self) -> &str {
            &self.tag
        }

        async fn handle_connection(
            &self,
            stream: BoxStream,
            _remote_addr: SocketAddr,
        ) -> Result<InboundResult> {
            let session = SessionContext::new(
                self.tag.clone(),
                crate::common::net::Destination::tcp(
                    crate::common::net::Address::Domain("example.com".into()),
                    80,
                ),
            );
            Ok(InboundResult { stream, session })
        }
    }

    struct MockOutboundHandler {
        tag: String,
    }

    #[async_trait]
    impl OutboundHandler for MockOutboundHandler {
        fn tag(&self) -> &str {
            &self.tag
        }

        async fn connect(&self, _session: &SessionContext) -> Result<BoxStream> {
            let (client, _server) = duplex(1024);
            Ok(Box::pin(client))
        }
    }

    struct TagPrefixSelector {
        prefix: String,
    }

    impl HandlerSelector for TagPrefixSelector {
        fn select(&self, tags: &[String]) -> Vec<String> {
            tags.iter()
                .filter(|t| t.starts_with(&self.prefix))
                .cloned()
                .collect()
        }
    }

    #[tokio::test]
    async fn test_inbound_manager_lifecycle_and_lookup() {
        let manager = DefaultInboundManager::new();

        let h1 = Arc::new(MockInboundHandler {
            tag: "in-http".into(),
        });
        let h2 = Arc::new(MockInboundHandler {
            tag: "in-socks".into(),
        });

        manager.add_handler(h1).await.unwrap();
        manager.add_handler(h2).await.unwrap();

        assert_eq!(manager.list_handlers().await.len(), 2);

        let retrieved = manager.get_handler("in-http").await.unwrap();
        assert_eq!(retrieved.tag(), "in-http");

        manager.remove_handler("in-http").await.unwrap();
        assert_eq!(manager.list_handlers().await.len(), 1);
        assert!(manager.get_handler("in-http").await.is_err());
    }

    #[tokio::test]
    async fn test_outbound_manager_default_and_selector() {
        let manager = DefaultOutboundManager::new();

        let h1 = Arc::new(MockOutboundHandler {
            tag: "proxy-us".into(),
        });
        let h2 = Arc::new(MockOutboundHandler {
            tag: "proxy-jp".into(),
        });
        let h3 = Arc::new(MockOutboundHandler {
            tag: "direct".into(),
        });

        manager.add_handler(h1).await.unwrap();
        manager.add_handler(h2).await.unwrap();
        manager.add_handler(h3).await.unwrap();

        let default = manager.get_default_handler().await.unwrap();
        assert_eq!(default.tag(), "proxy-us");

        let selector = TagPrefixSelector {
            prefix: "proxy-".into(),
        };
        let selected = manager.select_handlers(&selector).await;
        assert_eq!(selected.len(), 2);
    }
}
