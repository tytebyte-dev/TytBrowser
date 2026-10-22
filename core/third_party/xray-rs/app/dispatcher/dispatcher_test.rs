// Module: app\dispatcher\dispatcher_test.rs
// Comprehensive unit tests for DefaultDispatcher, Sniffer, Override logic, and Stats

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::net::{IpAddr, Ipv4Addr};
    use std::sync::Arc;
    use tokio::io::AsyncWriteExt;

    use crate::app::dispatcher::default::DefaultDispatcher;
    use crate::app::dispatcher::sniffer::{SniffResult, Sniffer};
    use crate::app::dns::fakedns::FakeDnsHolder;
    use crate::common::errors::Result;
    use crate::common::net::{Address, BoxStream, Destination, Network};
    use crate::common::protocol::{SessionContext, User};
    use crate::common::session::SniffingRequest;
    use crate::features::feature::Feature;
    use crate::features::outbound::OutboundHandler;
    use crate::features::policy::{PolicyManager, SessionPolicy, SystemPolicy};
    use crate::features::routing::RouterFeature;
    use crate::features::stats::DefaultStatsManager;

    struct MockRouter {
        picked: String,
    }

    impl RouterFeature for MockRouter {
        fn pick_outbound(&self, _session: &SessionContext) -> Option<&str> {
            Some(&self.picked)
        }
    }

    struct MockEchoOutbound {
        tag: String,
    }

    #[async_trait]
    impl OutboundHandler for MockEchoOutbound {
        fn tag(&self) -> &str {
            &self.tag
        }

        async fn connect(&self, _session: &SessionContext) -> Result<BoxStream> {
            let (client, mut server) = tokio::io::duplex(4096);
            tokio::spawn(async move {
                let mut buf = [0u8; 1024];
                while let Ok(n) = tokio::io::AsyncReadExt::read(&mut server, &mut buf).await {
                    if n == 0 {
                        break;
                    }
                    if server.write_all(&buf[..n]).await.is_err() {
                        break;
                    }
                }
            });
            Ok(Box::pin(client))
        }
    }

    struct MockPolicyManager;
    impl Feature for MockPolicyManager {
        fn feature_type(&self) -> &'static str {
            "policy_manager"
        }
    }
    impl PolicyManager for MockPolicyManager {
        fn for_level(&self, _level: u32) -> SessionPolicy {
            let mut p = SessionPolicy::default();
            p.stats.user_uplink = true;
            p.stats.user_downlink = true;
            p.stats.user_online = true;
            p
        }
        fn for_system(&self) -> SystemPolicy {
            SystemPolicy::default()
        }
    }

    #[test]
    fn test_sniffer_tls_client_hello() {
        // Construct minimal TLS ClientHello with SNI "xray.com"
        let domain = b"xray.com";
        let mut sni_ext = Vec::new();
        sni_ext.extend_from_slice(&[0x00, 0x00]); // Extension: server_name (0)
        let sni_list_len = (domain.len() + 3) as u16;
        let ext_len = (domain.len() + 5) as u16;
        sni_ext.extend_from_slice(&ext_len.to_be_bytes());
        sni_ext.extend_from_slice(&sni_list_len.to_be_bytes());
        sni_ext.push(0x00); // HostName type
        sni_ext.extend_from_slice(&(domain.len() as u16).to_be_bytes());
        sni_ext.extend_from_slice(domain);

        let mut payload = Vec::new();
        payload.extend_from_slice(&[0x16, 0x03, 0x01, 0x00, 0x00]); // TLS Record Header
        payload.push(0x01); // ClientHello
        payload.extend_from_slice(&[0x00, 0x00, 0x00]); // Length placeholder
        payload.extend_from_slice(&[0x03, 0x03]); // Version TLS 1.2
        payload.extend_from_slice(&[0u8; 32]); // Random
        payload.push(0x00); // Session ID len
        payload.extend_from_slice(&[0x00, 0x02, 0x13, 0x01]); // Ciphers (1 suite)
        payload.extend_from_slice(&[0x01, 0x00]); // Compression methods
        payload.extend_from_slice(&(sni_ext.len() as u16).to_be_bytes()); // Ext len
        payload.extend_from_slice(&sni_ext);

        let sniffer = Sniffer::new(None);
        let res = sniffer
            .sniff(&payload, Network::Tcp)
            .expect("Sniff should succeed");
        assert_eq!(res.protocol, "tls");
        assert_eq!(res.domain, "xray.com");
    }

    #[test]
    fn test_sniffer_http_host() {
        let http_req =
            b"GET /index.html HTTP/1.1\r\nHost: api.xray.io\r\nUser-Agent: curl/7.68.0\r\n\r\n";
        let sniffer = Sniffer::new(None);
        let res = sniffer
            .sniff(http_req, Network::Tcp)
            .expect("HTTP sniff should succeed");
        assert_eq!(res.protocol, "http");
        assert_eq!(res.domain, "api.xray.io");
    }

    #[test]
    fn test_sniffer_bittorrent() {
        let mut bt_req = Vec::new();
        bt_req.push(19);
        bt_req.extend_from_slice(b"BitTorrent protocol");
        bt_req.extend_from_slice(&[0u8; 8]); // reserved
        bt_req.extend_from_slice(&[1u8; 20]); // info hash
        bt_req.extend_from_slice(&[2u8; 20]); // peer id

        let sniffer = Sniffer::new(None);
        let res = sniffer
            .sniff(&bt_req, Network::Tcp)
            .expect("BitTorrent sniff should succeed");
        assert_eq!(res.protocol, "bittorrent");
    }

    #[test]
    fn test_should_override_logic() {
        let fake_holder = Arc::new(FakeDnsHolder::new("198.18.0.0/15").unwrap());
        let mut outbounds = HashMap::new();
        outbounds.insert(
            "proxy".into(),
            Arc::new(MockEchoOutbound {
                tag: "proxy".into(),
            }) as Arc<dyn OutboundHandler>,
        );
        let router = Arc::new(MockRouter {
            picked: "proxy".into(),
        });
        let dispatcher =
            DefaultDispatcher::new(outbounds, router).with_fakedns(fake_holder.clone());

        let req = SniffingRequest {
            enabled: true,
            metadata_only: false,
            route_only: false,
            exclude_for_domain: vec!["regexp:^.*\\.apple\\.com$".into(), "blocked.com".into()],
            override_destination_for_protocol: vec!["tls".into(), "http".into(), "fakedns".into()],
        };

        let dest_ip = Destination::new(Address::ip(IpAddr::V4(Ipv4Addr::new(198, 18, 0, 10))), 443);

        // 1. Should override normal TLS
        let res_tls = SniffResult {
            protocol: "tls".into(),
            domain: "google.com".into(),
        };
        assert!(dispatcher.should_override(&res_tls, &req, &dest_ip));

        // 2. Should NOT override regex matched domain
        let res_apple = SniffResult {
            protocol: "tls".into(),
            domain: "service.apple.com".into(),
        };
        assert!(!dispatcher.should_override(&res_apple, &req, &dest_ip));

        // 3. Should NOT override exact excluded domain
        let res_blocked = SniffResult {
            protocol: "http".into(),
            domain: "blocked.com".into(),
        };
        assert!(!dispatcher.should_override(&res_blocked, &req, &dest_ip));

        // 4. FakeDNS pool check
        let res_fakedns = SniffResult {
            protocol: "fakedns".into(),
            domain: "example.org".into(),
        };
        assert!(dispatcher.should_override(&res_fakedns, &req, &dest_ip));

        // Outside pool destination
        let dest_outside =
            Destination::new(Address::ip(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))), 443);
        let req_fake_only = SniffingRequest {
            enabled: true,
            metadata_only: false,
            route_only: false,
            exclude_for_domain: vec![],
            override_destination_for_protocol: vec!["fakedns".into()],
        };
        let res_unknown = SniffResult {
            protocol: "unknown".into(),
            domain: "example.org".into(),
        };
        assert!(!dispatcher.should_override(&res_unknown, &req_fake_only, &dest_outside));
    }

    #[tokio::test]
    async fn test_dispatcher_routing_and_stats() {
        let mut outbounds = HashMap::new();
        outbounds.insert(
            "direct".into(),
            Arc::new(MockEchoOutbound {
                tag: "direct".into(),
            }) as Arc<dyn OutboundHandler>,
        );
        let router = Arc::new(MockRouter {
            picked: "direct".into(),
        });
        let stats_mgr = Arc::new(DefaultStatsManager::new());
        let policy_mgr = Arc::new(MockPolicyManager);

        let dispatcher = DefaultDispatcher::new(outbounds, router)
            .with_stats(stats_mgr.clone())
            .with_policy(policy_mgr);

        let (client_stream, server_stream) = tokio::io::duplex(4096);

        let dest = Destination::new(Address::Domain("echo.service".into()), 80);
        let user = User {
            id: uuid::Uuid::new_v4(),
            email: "alice@example.com".into(),
            level: 0,
        };
        let mut session = SessionContext::new("inbound-http", dest).with_user(user);
        session.source = Some("127.0.0.1:54321".parse().unwrap());

        // Spawn dispatch in background
        let disp_handle =
            tokio::spawn(
                async move { dispatcher.dispatch(Box::pin(server_stream), session).await },
            );

        // Write some payload through client_stream and read back echo
        let mut client_pinned = Box::pin(client_stream);
        let msg = b"Hello, Xray Dispatcher!";
        client_pinned.write_all(msg).await.unwrap();

        let mut buf = vec![0u8; msg.len()];
        tokio::io::AsyncReadExt::read_exact(&mut client_pinned, &mut buf)
            .await
            .unwrap();
        assert_eq!(&buf, msg);

        // Drop client stream to terminate echo loop
        drop(client_pinned);
        let res = disp_handle.await.unwrap();
        assert!(res.is_ok());

        // Check stats counter
        let up_counter = stats_mgr
            .get_counter("user>>>alice@example.com>>>traffic>>>uplink")
            .expect("Uplink counter exists");
        let down_counter = stats_mgr
            .get_counter("user>>>alice@example.com>>>traffic>>>downlink")
            .expect("Downlink counter exists");
        assert_eq!(up_counter.value(), msg.len() as i64);
        assert_eq!(down_counter.value(), msg.len() as i64);
    }
}
