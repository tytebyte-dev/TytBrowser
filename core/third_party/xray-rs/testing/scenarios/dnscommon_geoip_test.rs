#[cfg(test)]
mod tests {
    use crate::app::dns::{IPRecord, QuicNameServer, fqdn};
    use crate::app::router::condition_geoip::{Cidr, GeoIpMatcher, HeuristicGeoIpMatcher};
    use std::net::{IpAddr, Ipv4Addr};
    use std::time::Duration;

    #[test]
    fn test_dnscommon_fqdn_and_ip_record() {
        assert_eq!(fqdn("google.com"), "google.com.");
        assert_eq!(fqdn("google.com."), "google.com.");

        let ip = IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8));
        let record = IPRecord::new(1234, vec![ip], Duration::from_secs(60), 0);
        assert!(!record.is_expired());
        assert_eq!(record.get_ips().unwrap(), &[ip]);
    }

    #[test]
    fn test_quic_nameserver_creation() {
        let doq = QuicNameServer::new("quic.dns.resolver:853");
        assert_eq!(doq.url(), "quic.dns.resolver:853");
    }

    #[test]
    fn test_heuristic_geoip_matcher() {
        let net1 = Cidr::parse("10.0.0.0/8").unwrap();
        let net2 = Cidr::parse("192.168.0.0/16").unwrap();
        let mut matcher = HeuristicGeoIpMatcher::new(vec![net1, net2], false);

        let ip_match = IpAddr::V4(Ipv4Addr::new(10, 1, 2, 3));
        let ip_unmatch = IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8));

        assert!(matcher.matches_ip(&ip_match));
        assert!(!matcher.matches_ip(&ip_unmatch));

        let (m, u) = matcher.filter_ips(&[ip_match, ip_unmatch]);
        assert_eq!(m.len(), 1);
        assert_eq!(u.len(), 1);

        matcher.toggle_reverse();
        assert!(!matcher.matches_ip(&ip_match));
        assert!(matcher.matches_ip(&ip_unmatch));
    }
}
