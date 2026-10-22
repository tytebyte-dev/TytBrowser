// Module: common\net\address_test.rs
// 1:1 Rust unit test suite corresponding to Go common\net\address_test.go

#[cfg(test)]
mod tests {
    use super::super::address::{
        Address, AddressFamily, any_ip, any_ipv6, ip_address_from_bytes, local_host_domain,
        local_host_ip, local_host_ipv6, parse_address,
    };
    use std::net::{Ipv4Addr, Ipv6Addr};
    use std::str::FromStr;

    #[test]
    fn test_address_properties() {
        let a1 = Address::Ipv4(Ipv4Addr::new(1, 2, 3, 4));
        assert!(a1.is_ip());
        assert!(a1.is_ipv4());
        assert!(!a1.is_ipv6());
        assert!(!a1.is_domain());
        assert_eq!(a1.family(), AddressFamily::IPv4);
        assert_eq!(a1.to_string(), "1.2.3.4");

        let a2 = Address::Ipv6(Ipv6Addr::new(0x2001, 0x4860, 0, 0x2001, 0, 0, 0, 0x68));
        assert!(a2.is_ip());
        assert!(a2.is_ipv6());
        assert!(!a2.is_ipv4());
        assert!(!a2.is_domain());
        assert_eq!(a2.family(), AddressFamily::IPv6);
        assert_eq!(a2.to_string(), "[2001:4860:0:2001::68]");

        let a3 = Address::Domain("example.com".into());
        assert!(!a3.is_ip());
        assert!(a3.is_domain());
        assert_eq!(a3.family(), AddressFamily::Domain);
        assert_eq!(a3.domain_name(), Some("example.com"));
        assert_eq!(a3.to_string(), "example.com");
    }

    #[test]
    fn test_address_parse_go_parity() {
        let a1 = Address::from_str("1.2.3.4").unwrap();
        assert_eq!(a1, Address::Ipv4(Ipv4Addr::new(1, 2, 3, 4)));

        let a2 = Address::from_str("[2001:4860:0:2001::68]").unwrap();
        assert_eq!(
            a2,
            Address::Ipv6(Ipv6Addr::new(0x2001, 0x4860, 0, 0x2001, 0, 0, 0, 0x68))
        );

        let a3 = Address::from_str("example.com").unwrap();
        assert_eq!(a3, Address::Domain("example.com".into()));

        // Test IPv4-mapped in IPv6
        let a4 = parse_address("[::ffff:123.151.71.143]");
        assert_eq!(a4.family(), AddressFamily::IPv4);
        assert_eq!(a4.to_string(), "123.151.71.143");

        let a5 =
            ip_address_from_bytes(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 1, 2, 3, 4]).unwrap();
        assert_eq!(a5.family(), AddressFamily::IPv4);
        assert_eq!(a5.to_string(), "1.2.3.4");

        // Constants
        assert_eq!(local_host_ip().to_string(), "127.0.0.1");
        assert_eq!(any_ip().to_string(), "0.0.0.0");
        assert_eq!(local_host_domain().to_string(), "localhost");
        assert_eq!(local_host_ipv6().to_string(), "[::1]");
        assert_eq!(any_ipv6().to_string(), "[::]");
    }
}
