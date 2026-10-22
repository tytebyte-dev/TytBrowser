// Module: common\strmatcher\full_matcher_test.rs
// 1:1 Rust unit test suite corresponding to Go common\strmatcher\full_matcher_test.go

#[cfg(test)]
mod tests {
    use super::super::full_matcher::{FullMatcher, FullMatcherGroup};
    use super::super::strmatcher::Matcher;

    #[test]
    fn test_full_matcher() {
        let matcher = FullMatcher::new("v2ray.com");
        assert!(matcher.match_str("v2ray.com"));
        assert!(!matcher.match_str("www.v2ray.com"));
        assert!(!matcher.match_str("google.com"));
    }

    #[test]
    fn test_full_matcher_group() {
        let mut g = FullMatcherGroup::new();
        g.add("example.com", 1);
        g.add("google.com", 2);
        g.add("x.a.com", 3);
        g.add("x.y.com", 4);
        g.add("x.y.com", 6);

        let test_cases: Vec<(&str, Vec<u32>)> = vec![
            ("example.com", vec![1]),
            ("y.com", vec![]),
            ("x.y.com", vec![4, 6]),
        ];

        for (domain, expected) in test_cases {
            let r = g.match_str(domain);
            assert_eq!(r, expected, "failed domain: {}", domain);
        }
    }

    #[test]
    fn test_empty_full_matcher_group() {
        let g = FullMatcherGroup::new();
        let r = g.match_str("example.com");
        assert!(r.is_empty());
    }
}
