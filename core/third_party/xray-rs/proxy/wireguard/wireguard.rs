// Module: proxy\wireguard\wireguard.rs
// 1:1 Rust implementation corresponding to Go proxy\wireguard\wireguard.go

use super::config::WireGuardConfig;
use crate::common::errors::{Error, Result};
use std::net::IpAddr;
use std::str::FromStr;

pub const PROTOCOL_NAME: &str = "wireguard";
pub const DEFAULT_MTU: u32 = 1420;

/// Parses endpoint/interface addresses and validates /32 for IPv4 and /128 for IPv6
pub fn parse_endpoints(endpoints: &[String]) -> Result<(Vec<IpAddr>, bool, bool)> {
    let mut addrs = Vec::with_capacity(endpoints.len());
    let mut has_ipv4 = false;
    let mut has_ipv6 = false;

    for s in endpoints {
        let addr = if let Some((ip_str, prefix_str)) = s.split_once('/') {
            let ip = IpAddr::from_str(ip_str)
                .map_err(|e| Error::Config(format!("invalid IP: {}", e)))?;
            let prefix: u8 = prefix_str
                .parse()
                .map_err(|_| Error::Config("invalid subnet prefix".into()))?;
            match ip {
                IpAddr::V4(_) => {
                    if prefix != 32 {
                        return Err(Error::Config(
                            "interface address subnet should be /32 for IPv4".into(),
                        ));
                    }
                }
                IpAddr::V6(_) => {
                    if prefix != 128 {
                        return Err(Error::Config(
                            "interface address subnet should be /128 for IPv6".into(),
                        ));
                    }
                }
            }
            ip
        } else {
            IpAddr::from_str(s).map_err(|e| Error::Config(format!("invalid IP: {}", e)))?
        };

        match addr {
            IpAddr::V4(_) => has_ipv4 = true,
            IpAddr::V6(_) => has_ipv6 = true,
        }
        addrs.push(addr);
    }

    Ok((addrs, has_ipv4, has_ipv6))
}

/// Serializes configuration into standard WireGuard IPC/UAPI request format
pub fn create_ipc_request(
    conf: &WireGuardConfig,
    is_client: bool,
    listen_port: Option<u16>,
) -> String {
    let mut request = String::new();
    request.push_str(&format!("private_key={}\n", conf.secret_key));

    if !is_client {
        let port = listen_port.unwrap_or(1337);
        request.push_str(&format!("listen_port={}\n", port));
    }

    for peer in &conf.peers {
        if !peer.public_key.is_empty() {
            request.push_str(&format!("public_key={}\n", peer.public_key));
        }

        if !peer.endpoint.is_empty() {
            request.push_str(&format!("endpoint={}\n", peer.endpoint));
        }

        if peer.keepalive != 0 {
            request.push_str(&format!(
                "persistent_keepalive_interval={}\n",
                peer.keepalive
            ));
        }
    }

    request
}

#[cfg(test)]
mod tests {
    use super::super::config::WireGuardPeer;
    use super::*;

    #[test]
    fn test_parse_endpoints() {
        let eps = vec![
            "10.0.0.2/32".to_string(),
            "fd00::2/128".to_string(),
            "1.1.1.1".to_string(),
        ];
        let (parsed, has_v4, has_v6) = parse_endpoints(&eps).expect("valid endpoints");
        assert_eq!(parsed.len(), 3);
        assert!(has_v4);
        assert!(has_v6);

        // Invalid prefix
        let bad = vec!["10.0.0.2/24".to_string()];
        assert!(parse_endpoints(&bad).is_err());
    }

    #[test]
    fn test_create_ipc_request() {
        let conf = WireGuardConfig {
            secret_key: "CLIENT_SECRET_KEY".to_string(),
            address: vec!["10.0.0.2/32".to_string()],
            peers: vec![WireGuardPeer {
                public_key: "SERVER_PUBLIC_KEY".to_string(),
                endpoint: "198.51.100.1:51820".to_string(),
                keepalive: 25,
            }],
            mtu: Some(1420),
            reserved: None,
        };

        let ipc = create_ipc_request(&conf, true, None);
        assert!(ipc.contains("private_key=CLIENT_SECRET_KEY"));
        assert!(ipc.contains("public_key=SERVER_PUBLIC_KEY"));
        assert!(ipc.contains("endpoint=198.51.100.1:51820"));
        assert!(ipc.contains("persistent_keepalive_interval=25"));
        assert!(!ipc.contains("listen_port="));

        let server_ipc = create_ipc_request(&conf, false, Some(51820));
        assert!(server_ipc.contains("listen_port=51820"));
    }
}
