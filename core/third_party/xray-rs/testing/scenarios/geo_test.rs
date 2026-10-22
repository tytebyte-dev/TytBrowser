#[cfg(test)]
mod tests {
    use crate::app::router::condition::{DomainMatcher, IpMatcher};
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_geosite_and_geoip_matching() {
        let google_matcher = DomainMatcher::parse("geosite:google");
        assert!(google_matcher.matches("www.google.com"));
        assert!(google_matcher.matches("youtube.com"));
        assert!(!google_matcher.matches("baidu.com"));

        let cn_matcher = DomainMatcher::parse("geosite:cn");
        assert!(cn_matcher.matches("www.baidu.com"));
        assert!(cn_matcher.matches("taobao.com"));
        assert!(!cn_matcher.matches("google.com"));

        let private_ip = IpMatcher::parse("geoip:private").unwrap();
        assert!(private_ip.matches(&IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100))));
        assert!(private_ip.matches(&IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
        assert!(private_ip.matches(&IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))));
        assert!(!private_ip.matches(&IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))));

        let cidr_ip = IpMatcher::parse("172.18.0.0/16").unwrap();
        assert!(cidr_ip.matches(&IpAddr::V4(Ipv4Addr::new(172, 18, 10, 5))));
        assert!(!cidr_ip.matches(&IpAddr::V4(Ipv4Addr::new(172, 19, 10, 5))));
    }
}
