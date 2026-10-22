// Module: app\dns\hosts_test.rs
// 1:1 Rust unit test suite corresponding to Go app\dns\hosts_test.go

#[cfg(test)]
mod tests {
    use super::super::hosts::StaticHosts;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_static_hosts_exact_and_domain() {
        let hosts = StaticHosts::new();
        let target_ip: IpAddr = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));

        hosts.add_exact("example.com", vec![target_ip]);
        let res = hosts.lookup("example.com");
        assert_eq!(res, Some(vec![target_ip]));

        let not_found = hosts.lookup("google.com");
        assert_eq!(not_found, None);
    }
}
