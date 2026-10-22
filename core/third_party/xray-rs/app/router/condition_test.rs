// Module: app\router\condition_test.rs
// 1:1 Rust unit test suite corresponding to Go app\router\condition_test.go

#[cfg(test)]
mod tests {
    use super::super::condition::{DomainMatcher, IpMatcher};
    use std::net::Ipv4Addr;

    #[test]
    fn test_domain_matcher_rules() {
        let exact = DomainMatcher::Exact("google.com".into());
        assert!(exact.matches("google.com"));
        assert!(!exact.matches("mail.google.com"));

        let suffix = DomainMatcher::Suffix("google.com".into());
        assert!(suffix.matches("google.com"));
        assert!(suffix.matches("mail.google.com"));
        assert!(!suffix.matches("notgoogle.com"));
    }

    #[test]
    fn test_ip_matcher_exact() {
        let matcher = IpMatcher::Exact(std::net::IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));
        assert!(matcher.matches(&std::net::IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))));
        assert!(!matcher.matches(&std::net::IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2))));
    }
}
