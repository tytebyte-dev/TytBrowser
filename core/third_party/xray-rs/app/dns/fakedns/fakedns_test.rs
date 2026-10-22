// Module: app\dns\fakedns\fakedns_test.rs
// 1:1 Rust unit test suite corresponding to Go app\dns\fakedns\fakedns_test.go

#[cfg(test)]
mod tests {
    use super::super::fake::FakeDnsHolder;
    use std::net::IpAddr;

    #[test]
    fn test_fakedns_domain_ip_mapping() {
        let holder = FakeDnsHolder::new("198.18.0.0/15").unwrap();

        let ip1 = holder.get_fake_ip_for_domain("example.com");
        assert!(holder.is_fake_ip(&IpAddr::V4(ip1)));

        let ip2 = holder.get_fake_ip_for_domain("example.com");
        assert_eq!(ip1, ip2);

        let domain = holder.get_domain_for_fake_ip(&ip1);
        assert_eq!(domain, Some("example.com".to_string()));
    }
}
