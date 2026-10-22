// Module: common\strmatcher\strmatcher_test.rs
// 1:1 Rust unit test suite corresponding to Go common\strmatcher\strmatcher_test.go

#[cfg(test)]
mod tests {
    use super::super::ac_automaton_matcher::AcAutomatonMatcher;
    use super::super::strmatcher::{MatcherGroup, MatcherType};

    #[test]
    fn test_matcher_group() {
        let rules = vec![
            (MatcherType::Regex, "apis\\.us$"),
            (MatcherType::Substr, "apis"),
            (MatcherType::Domain, "googleapis.com"),
            (MatcherType::Domain, "com"),
            (MatcherType::Full, "www.baidu.com"),
            (MatcherType::Substr, "apis"),
            (MatcherType::Domain, "googleapis.com"),
            (MatcherType::Full, "fonts.googleapis.com"),
            (MatcherType::Full, "www.baidu.com"),
            (MatcherType::Domain, "example.com"),
        ];

        let cases = vec![
            ("www.baidu.com", vec![5, 9, 4]),
            ("fonts.googleapis.com", vec![8, 3, 7, 4, 2, 6]),
            ("example.googleapis.com", vec![3, 7, 4, 2, 6]),
            ("testapis.us", vec![1, 2, 6]),
            ("example.com", vec![10, 4]),
        ];

        let mut group = MatcherGroup::new();
        for (m_type, domain) in rules {
            let matcher = m_type.new_matcher(domain).unwrap();
            group.add(matcher);
        }

        for (input, expected) in cases {
            let m = group.match_pattern(input);
            assert_eq!(m, expected, "input: {}", input);
        }
    }

    #[test]
    fn test_ac_automaton() {
        let cases1 = vec![
            (
                "xtls.github.io",
                MatcherType::Domain,
                "www.xtls.github.io",
                true,
            ),
            (
                "xtls.github.io",
                MatcherType::Domain,
                "xtls.github.io",
                true,
            ),
            (
                "xtls.github.io",
                MatcherType::Domain,
                "www.xtis.github.io",
                false,
            ),
            (
                "xtls.github.io",
                MatcherType::Domain,
                "tls.github.io",
                false,
            ),
            (
                "xtls.github.io",
                MatcherType::Domain,
                "xxtls.github.io",
                false,
            ),
            ("xtls.github.io", MatcherType::Full, "xtls.github.io", true),
            (
                "xtls.github.io",
                MatcherType::Full,
                "xxtls.github.io",
                false,
            ),
        ];

        for (pattern, m_type, input, expected) in cases1 {
            let mut ac = AcAutomatonMatcher::new();
            ac.add(pattern, m_type);
            ac.build();
            assert_eq!(ac.match_str(input), expected, "input: {}", input);
        }
    }
}
