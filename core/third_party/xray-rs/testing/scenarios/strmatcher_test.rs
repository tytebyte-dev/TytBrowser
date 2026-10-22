#[cfg(test)]
mod tests {
    use crate::common::strmatcher::{
        DomainMatcher, DomainMatcherGroup, FullMatcher, Matcher, RegexMatcher, SubstrMatcher,
    };

    #[test]
    fn test_full_matcher() {
        let matcher = FullMatcher::new("google.com");
        assert!(matcher.match_str("google.com"));
        assert!(!matcher.match_str("www.google.com"));
    }

    #[test]
    fn test_substr_matcher() {
        let matcher = SubstrMatcher::new("youtube");
        assert!(matcher.match_str("www.youtube.com"));
        assert!(matcher.match_str("m.youtube.com/watch"));
        assert!(!matcher.match_str("www.google.com"));
    }

    #[test]
    fn test_domain_matcher() {
        let matcher = DomainMatcher::new("twitter.com");
        assert!(matcher.match_str("twitter.com"));
        assert!(matcher.match_str("api.twitter.com"));
        assert!(matcher.match_str("sub.api.TWITTER.com"));
        assert!(!matcher.match_str("nottwitter.com"));
    }

    #[test]
    fn test_regex_matcher() {
        let matcher = RegexMatcher::new(r"^https?://.*\.org$").unwrap();
        assert!(matcher.match_str("https://wikipedia.org"));
        assert!(!matcher.match_str("https://wikipedia.com"));
    }

    #[test]
    fn test_domain_matcher_group_trie() {
        let mut group = DomainMatcherGroup::new();
        group.add("com", 1);
        group.add("google.com", 2);
        group.add("mail.google.com", 3);

        let matches = group.match_domain("mail.google.com");
        assert_eq!(matches, vec![3, 2, 1]);

        let matches_google = group.match_domain("drive.google.com");
        assert_eq!(matches_google, vec![2, 1]);

        let matches_other = group.match_domain("bing.com");
        assert_eq!(matches_other, vec![1]);

        let matches_none = group.match_domain("baidu.cn");
        assert!(matches_none.is_empty());
    }
}
