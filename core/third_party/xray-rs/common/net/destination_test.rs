// Module: common\net\destination_test.rs
// 1:1 Rust unit test suite corresponding to Go common\net\destination_test.go

#[cfg(test)]
mod tests {
    use super::super::destination::{parse_destination, tcp_destination, udp_destination};
    use super::super::{Address, Destination, Network};
    use std::net::{Ipv4Addr, SocketAddr};
    use std::str::FromStr;

    #[test]
    fn test_destination_construction() {
        let dest = Destination::tcp(Address::Ipv4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        assert_eq!(dest.port, 8080);
        assert_eq!(dest.network, Network::Tcp);
        assert_eq!(dest.to_string(), "127.0.0.1:8080");
        assert_eq!(dest.net_addr(), "127.0.0.1:8080");

        let dest_udp = Destination::udp(Address::Domain("example.com".into()), 53);
        assert_eq!(dest_udp.port, 53);
        assert_eq!(dest_udp.network, Network::Udp);
        assert_eq!(dest_udp.to_string(), "example.com:53");
        assert_eq!(dest_udp.net_addr(), "example.com:53");
    }

    #[test]
    fn test_destination_helpers_and_parse() {
        let d = tcp_destination(Address::Ipv4(Ipv4Addr::new(1, 2, 3, 4)), 80);
        assert_eq!(d.net_addr(), "1.2.3.4:80");
        assert_eq!(d.network, Network::Tcp);

        let d_udp = udp_destination(Address::Domain("example.com".into()), 53);
        assert_eq!(d_udp.net_addr(), "example.com:53");
        assert_eq!(d_udp.network, Network::Udp);

        let parsed_tcp = parse_destination("tcp:1.2.3.4:80").unwrap();
        assert_eq!(parsed_tcp.network, Network::Tcp);
        assert_eq!(parsed_tcp.net_addr(), "1.2.3.4:80");

        let parsed_udp = parse_destination("udp:8.8.8.8:53").unwrap();
        assert_eq!(parsed_udp.network, Network::Udp);
        assert_eq!(parsed_udp.net_addr(), "8.8.8.8:53");
    }

    #[test]
    fn test_destination_from_socket_addr() {
        let sa: SocketAddr = "192.168.1.1:443".parse().unwrap();
        let dest = Destination::from(sa);
        assert_eq!(dest.port, 443);
        assert_eq!(dest.address, Address::Ipv4(Ipv4Addr::new(192, 168, 1, 1)));
        assert_eq!(dest.to_socket_addr(), Some(sa));
    }

    #[test]
    fn test_destination_from_str() {
        let d1 = Destination::from_str("1.1.1.1:853").unwrap();
        assert_eq!(d1.port, 853);
        assert_eq!(d1.address, Address::Ipv4(Ipv4Addr::new(1, 1, 1, 1)));

        let d2 = Destination::from_str("[::1]:80").unwrap();
        assert_eq!(d2.port, 80);
        assert_eq!(d2.address, Address::Ipv6(std::net::Ipv6Addr::LOCALHOST));
    }
}
