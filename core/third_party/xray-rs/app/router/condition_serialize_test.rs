// Module: app\\router\\condition_serialize_test.rs
// 1:1 Rust unit test suite corresponding to Go app\\router\\condition_serialize_test.go

#[cfg(test)]
mod tests {
    use super::super::condition::DomainMatcher;

    #[test]
    fn test_domain_condition_match() {
        let m = DomainMatcher::parse("domain:google.com");
        assert!(m.matches("www.google.com"));
        assert!(!m.matches("example.com"));
    }
}
