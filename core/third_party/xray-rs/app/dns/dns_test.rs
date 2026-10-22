// Module: app\dns\dns_test.rs
// 1:1 Rust unit test suite corresponding to Go app\dns\dns_test.go

#[cfg(test)]
mod tests {
    use super::super::dns::DnsClient;
    use std::net::{IpAddr, Ipv4Addr};

    #[tokio::test]
    async fn test_dns_client_lookup_hosts() {
        let client = DnsClient::new();
        let target_ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        client.add_host("custom.internal", vec![target_ip]);

        let ips = client.lookup_ip("custom.internal").await.unwrap();
        assert_eq!(ips, vec![target_ip]);
    }
}
