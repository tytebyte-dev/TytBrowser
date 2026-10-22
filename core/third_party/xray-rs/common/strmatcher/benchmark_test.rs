// Module: common\strmatcher\benchmark_test.rs
// 1:1 Rust unit test suite corresponding to Go common\strmatcher\benchmark_test.go

#[cfg(test)]
mod tests {
    use super::super::ac_automaton_matcher::AcAutomatonMatcher;
    use super::super::domain_matcher::DomainMatcherGroup;
    use super::super::full_matcher::FullMatcherGroup;
    use super::super::strmatcher::{MatcherGroup, MatcherType};

    #[test]
    fn test_strmatcher_benchmark_simulation() {
        let mut ac = AcAutomatonMatcher::new();
        for i in 1..=128 {
            ac.add(&format!("{}.xray.com", i), MatcherType::Domain);
        }
        ac.build();
        assert!(!ac.match_str("0.xray.com"));
        assert!(ac.match_str("1.xray.com"));

        let mut dg = DomainMatcherGroup::new();
        for i in 1..=128 {
            dg.add(&format!("{}.example.com", i), i as u32);
        }
        assert!(dg.match_domain("0.example.com").is_empty());
        assert_eq!(dg.match_domain("1.example.com"), vec![1]);

        let mut fg = FullMatcherGroup::new();
        for i in 1..=128 {
            fg.add(&format!("{}.example.com", i), i as u32);
        }
        assert!(fg.match_str("0.example.com").is_empty());
        assert_eq!(fg.match_str("1.example.com"), vec![1]);

        let mut mg = MatcherGroup::new();
        for i in 1..=128 {
            let m = MatcherType::Domain
                .new_matcher(&format!("{}.example.com", i))
                .unwrap();
            mg.add(m);
        }
        assert!(mg.match_pattern("0.example.com").is_empty());
        assert!(!mg.match_pattern("1.example.com").is_empty());
    }
}
