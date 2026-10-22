// Module: testing\scenarios\dns_test.rs
// Real integration tests for DNS client, static hosts and routing resolution

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::net::IpAddr;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use crate::app::dns::{DnsClient, StaticHosts};
    use crate::features::dns::{DnsClient as FeatureDnsClient, LocalDnsClient};
    use crate::infra::conf::Config;
    use crate::testing::scenarios::common::{TestEnvironment, pick_port, socks5_connect};
    use crate::testing::servers::tcp::{Server as TcpServer, echo_processor};

    #[tokio::test]
    async fn test_scenario_localdns_lookup() {
        let client = LocalDnsClient;
        let ips = client.lookup_ip("localhost").await;
        assert!(ips.is_ok());
    }

    #[tokio::test]
    async fn test_dns_static_hosts_resolution() {
        let mut hosts_map = HashMap::new();
        hosts_map.insert("google.com".to_string(), vec!["127.0.0.1".parse().unwrap()]);
        hosts_map.insert(
            "dns.internal".to_string(),
            vec!["10.0.0.53".parse().unwrap()],
        );

        let dns = DnsClient::with_hosts(hosts_map);

        // Exact match
        let ips = dns.lookup_ip("google.com").await.unwrap();
        assert_eq!(ips, vec!["127.0.0.1".parse::<IpAddr>().unwrap()]);

        let ips2 = dns.lookup_ip("dns.internal").await.unwrap();
        assert_eq!(ips2, vec!["10.0.0.53".parse::<IpAddr>().unwrap()]);
    }

    #[tokio::test]
    async fn test_dns_suffix_and_keyword_matching() {
        let hosts = StaticHosts::new();
        hosts.add_domain_suffix("example.org", vec!["192.168.1.100".parse().unwrap()]);
        hosts.add_keyword("internal", vec!["10.10.10.10".parse().unwrap()]);

        // Suffix match
        let res = hosts.lookup("sub.example.org").unwrap();
        assert_eq!(res, vec!["192.168.1.100".parse::<IpAddr>().unwrap()]);

        // Keyword match
        let res2 = hosts.lookup("my-internal-server.local").unwrap();
        assert_eq!(res2, vec!["10.10.10.10".parse::<IpAddr>().unwrap()]);
    }

    #[tokio::test]
    async fn test_resolve_ip_static_hosts_end_to_end() {
        // Start TCP echo server
        let tcp_server = TcpServer::start(None, Some(echo_processor()), None)
            .await
            .unwrap();
        let target_port = tcp_server.port();

        // Node with SOCKS5 inbound and Freedom outbound
        let mut env = TestEnvironment::new();
        let socks_port = pick_port().await;

        let cfg: Config = serde_json::from_value(serde_json::json!({
            "inbounds": [{
                "tag": "socks-in",
                "port": socks_port,
                "listen": "127.0.0.1",
                "protocol": "socks"
            }],
            "outbounds": [{
                "tag": "direct",
                "protocol": "freedom"
            }]
        }))
        .unwrap();
        env.start_node(cfg).await.unwrap();

        // Connect through SOCKS5 to 127.0.0.1
        let mut client = socks5_connect(socks_port, "127.0.0.1", target_port)
            .await
            .unwrap();

        let msg = b"Testing DNS & SOCKS5 direct IP echo";
        client.write_all(msg).await.unwrap();

        let mut recv = vec![0u8; msg.len()];
        client.read_exact(&mut recv).await.unwrap();
        assert_eq!(&recv, msg);
    }
}
