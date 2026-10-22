// Module: common\strmatcher\matchers_test.rs
// 1:1 Rust unit test suite corresponding to Go common\strmatcher\matchers_test.go

#[cfg(test)]
mod tests {
    use super::super::matchers::SubstrMatcher;
    use super::super::strmatcher::{Matcher, MatcherType};

    #[test]
    fn test_substr_matcher() {
        let matcher = SubstrMatcher::new("v2ray");
        assert!(matcher.match_str("v2ray.com"));
        assert!(matcher.match_str("www.v2ray.com"));
        assert!(!matcher.match_str("google.com"));
    }

    #[test]
    fn test_regex_matcher() {
        let m = MatcherType::Regex.new_matcher("^xray\\..*").unwrap();
        assert!(m.match_str("xray.com"));
        assert!(m.match_str("xray.org"));
        assert!(!m.match_str("myxray.com"));
    }

    #[test]
    fn test_matcher_matrix() {
        let cases = vec![
            ("example.com", MatcherType::Domain, "www.example.com", true),
            ("example.com", MatcherType::Domain, "example.com", true),
            ("example.com", MatcherType::Domain, "www.fxample.com", false),
            ("example.com", MatcherType::Domain, "xample.com", false),
            ("example.com", MatcherType::Domain, "xexample.com", false),
            ("example.com", MatcherType::Full, "example.com", true),
            ("example.com", MatcherType::Full, "xexample.com", false),
            ("example.com", MatcherType::Regex, "examplexcom", true),
        ];

        for (pattern, m_type, input, expected) in cases {
            let matcher = m_type.new_matcher(pattern).unwrap();
            assert_eq!(matcher.match_str(input), expected, "input: {}", input);
        }
    }
}
