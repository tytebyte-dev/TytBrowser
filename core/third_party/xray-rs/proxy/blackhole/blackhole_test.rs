// Module: proxy\blackhole\blackhole_test.rs
// 1:1 Rust unit test suite corresponding to Go proxy\blackhole\blackhole_test.go

#[cfg(test)]
mod tests {
    use super::super::blackhole::BlackholeHandler;
    use super::super::config::{BlackholeConfig, ResponseType};

    #[tokio::test]
    async fn test_blackhole_http_403_response() {
        let handler = BlackholeHandler::new(BlackholeConfig {
            response: ResponseType::Http403,
        });
        let mut buf = Vec::new();
        handler.handle(&mut buf).await.unwrap();
        assert!(buf.starts_with(b"HTTP/1.1 403 Forbidden"));
    }
}
