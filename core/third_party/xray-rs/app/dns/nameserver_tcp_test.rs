// Module: app\dns\nameserver_tcp_test.rs
// 1:1 Rust unit test suite corresponding to Go app\dns\nameserver_tcp_test.go

#[cfg(test)]
mod tests {
    use super::super::nameserver::NameServer;
    use super::super::nameserver_tcp::TcpNameServer;
    use std::net::SocketAddr;

    #[tokio::test]
    async fn test_tcp_nameserver_creation() {
        let addr: SocketAddr = "8.8.8.8:53".parse().unwrap();
        let ns = TcpNameServer::new(addr);
        assert_eq!(ns.name(), "tcp");
    }
}
