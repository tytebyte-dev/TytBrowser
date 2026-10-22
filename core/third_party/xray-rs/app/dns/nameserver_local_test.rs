// Module: app\dns\nameserver_local_test.rs
// 1:1 Rust unit test suite corresponding to Go app\dns\nameserver_local_test.go

#[cfg(test)]
mod tests {
    use super::super::nameserver::NameServer;
    use super::super::nameserver_local::LocalNameServer;

    #[tokio::test]
    async fn test_local_nameserver_resolution() {
        let ns = LocalNameServer::new();
        assert_eq!(ns.name(), "local");

        let res = ns.query_ip("localhost").await;
        assert!(res.is_ok());
        let ips = res.unwrap();
        assert!(!ips.is_empty());
    }
}
