// Module: testing\scenarios\proxyman_full_test.rs
// Test Proxyman OutboundManager lifecycle

#[cfg(test)]
mod tests {
    use crate::app::proxyman::DefaultOutboundManager;
    use crate::features::outbound::OutboundManager;
    use crate::proxy::freedom::Handler as FreedomHandler;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_proxyman_outbound_lifecycle() {
        let manager = DefaultOutboundManager::new();
        let freedom = Arc::new(FreedomHandler::new("freedom-direct"));

        manager.add_handler(freedom).await.unwrap();

        let handler = manager.get_handler("freedom-direct").await;
        assert!(handler.is_some());
        assert_eq!(handler.unwrap().tag(), "freedom-direct");

        let missing = manager.get_handler("non-existent").await;
        assert!(missing.is_none());
    }
}
