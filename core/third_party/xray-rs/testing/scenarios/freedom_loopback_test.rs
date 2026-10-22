// Module: testing\scenarios\freedom_loopback_test.rs
// Test Freedom and Loopback outbound handlers

#[cfg(test)]
mod tests {
    use crate::features::outbound::OutboundHandler;
    use crate::proxy::freedom::Handler as FreedomHandler;

    #[test]
    fn test_proxy_freedom_handler() {
        let handler = FreedomHandler::new("direct");
        assert_eq!(handler.tag(), "direct");
    }
}
