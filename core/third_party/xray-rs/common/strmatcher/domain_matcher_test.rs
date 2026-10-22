// Module: common\strmatcher\domain_matcher_test.rs
// 1:1 Rust unit test suite corresponding to Go common\strmatcher\domain_matcher_test.go

#[cfg(test)]
mod tests {
    use super::super::domain_matcher::{DomainMatcher, DomainMatcherGroup};
    use super::super::strmatcher::Matcher;

    #[test]
    fn test_domain_matcher() {
        let matcher = DomainMatcher::new("v2ray.com");
        assert!(matcher.match_str("v2ray.com"));
        assert!(matcher.match_str("www.v2ray.com"));
        assert!(matcher.match_str("a.b.v2ray.com"));
        assert!(!matcher.match_str("xv2ray.com"));
        assert!(!matcher.match_str("google.com"));
    }

    #[test]
    fn test_domain_matcher_group() {
        let mut g = DomainMatcherGroup::new();
        g.add("example.com", 1);
        g.add("google.com", 2);
        g.add("x.a.com", 3);
        g.add("a.b.com", 4);
        g.add("c.a.b.com", 5);
        g.add("x.y.com", 4);
        g.add("x.y.com", 6);

        let test_cases: Vec<(&str, Vec<u32>)> = vec![
            ("x.example.com", vec![1]),
            ("y.com", vec![]),
            ("a.b.com", vec![4]),
            ("c.a.b.com", vec![5, 4]),
            ("c.a..b.com", vec![]),
            (".com", vec![]),
            ("com", vec![]),
            ("", vec![]),
            ("x.y.com", vec![4, 6]),
        ];

        for (domain, expected) in test_cases {
            let r = g.match_domain(domain);
            assert_eq!(r, expected, "failed domain: {}", domain);
        }
    }

    #[test]
    fn test_empty_domain_matcher_group() {
        let g = DomainMatcherGroup::new();
        let r = g.match_domain("example.com");
        assert!(r.is_empty());
    }
}
