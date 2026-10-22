// Module: app\dns\nameserver_doh_test.rs
// 1:1 Rust unit test suite corresponding to Go app\dns\nameserver_doh_test.go

#[cfg(test)]
mod tests {
    use super::super::nameserver_doh::DohNameServer;

    #[test]
    fn test_doh_nameserver_creation() {
        let doh = DohNameServer::new("https://1.1.1.1/dns-query");
        assert_eq!(doh.url(), "https://1.1.1.1/dns-query");
    }
}
