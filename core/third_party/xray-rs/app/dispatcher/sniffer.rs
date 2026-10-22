use crate::app::dns::fakedns::FakeDnsHolder;
use crate::common::errors::{Error, Result};
use crate::common::net::Network;
use std::net::IpAddr;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SniffResult {
    pub protocol: String,
    pub domain: String,
}

pub struct Sniffer {
    fakedns: Option<Arc<FakeDnsHolder>>,
}

impl Sniffer {
    pub fn new(fakedns: Option<Arc<FakeDnsHolder>>) -> Self {
        Self { fakedns }
    }

    pub fn sniff_ip(&self, ip: IpAddr) -> Option<String> {
        if let Some(fdns) = &self.fakedns {
            match ip {
                IpAddr::V4(ref v4) => fdns.get_domain_for_fake_ip(v4),
                IpAddr::V6(_) => None,
            }
        } else {
            None
        }
    }

    pub fn is_in_fake_ip_pool(&self, ip: &IpAddr) -> bool {
        if let Some(fdns) = &self.fakedns {
            fdns.is_fake_ip(ip)
        } else {
            false
        }
    }

    pub fn sniff(&self, payload: &[u8], network: Network) -> Result<SniffResult> {
        // 1. Try TLS ClientHello SNI sniffing (for TCP)
        if network == Network::Tcp
            && payload.len() >= 5
            && payload[0] == 0x16
            && payload[1] == 0x03
            && let Some(domain) = Self::sniff_tls_sni(payload)
        {
            return Ok(SniffResult {
                protocol: "tls".into(),
                domain,
            });
        }

        // 2. Try HTTP Host sniffing (for TCP)
        if network == Network::Tcp
            && payload.len() >= 10
            && let Some(domain) = Self::sniff_http_host(payload)
        {
            return Ok(SniffResult {
                protocol: "http".into(),
                domain,
            });
        }

        // 3. Try QUIC SNI sniffing (for UDP)
        if network == Network::Udp
            && payload.len() >= 12
            && let Some(domain) = Self::sniff_quic_sni(payload)
        {
            return Ok(SniffResult {
                protocol: "quic".into(),
                domain,
            });
        }

        // 4. Try BitTorrent / uTP sniffing
        if let Ok(proto) = crate::common::protocol::bittorrent::BittorrentSniffer::sniff(payload) {
            return Ok(SniffResult {
                protocol: proto,
                domain: String::new(),
            });
        }

        Err(Error::Protocol("unknown content".into()))
    }

    pub fn sniff_tls_sni(payload: &[u8]) -> Option<String> {
        if payload.len() < 43 {
            return None;
        }

        let mut pos = 5; // Skip TLS Record Header
        if pos >= payload.len() || payload[pos] != 0x01 {
            return None; // Must be ClientHello
        }

        pos += 4; // Skip Handshake Header
        pos += 2 + 32; // Skip Version (2) + Random (32)
        if pos >= payload.len() {
            return None;
        }

        let session_id_len = payload[pos] as usize;
        pos += 1 + session_id_len;

        if pos + 2 > payload.len() {
            return None;
        }
        let cipher_suites_len = u16::from_be_bytes([payload[pos], payload[pos + 1]]) as usize;
        pos += 2 + cipher_suites_len;

        if pos + 1 > payload.len() {
            return None;
        }
        let comp_methods_len = payload[pos] as usize;
        pos += 1 + comp_methods_len;

        if pos + 2 > payload.len() {
            return None;
        }
        let extensions_len = u16::from_be_bytes([payload[pos], payload[pos + 1]]) as usize;
        pos += 2;

        let end_ext = (pos + extensions_len).min(payload.len());
        while pos + 4 <= end_ext {
            let ext_type = u16::from_be_bytes([payload[pos], payload[pos + 1]]);
            let ext_len = u16::from_be_bytes([payload[pos + 2], payload[pos + 3]]) as usize;
            pos += 4;

            if ext_type == 0x0000 {
                // Server Name Indication
                if pos + 5 <= end_ext {
                    let _server_name_list_len =
                        u16::from_be_bytes([payload[pos], payload[pos + 1]]);
                    let name_type = payload[pos + 2];
                    if name_type == 0 {
                        // HostName
                        let name_len =
                            u16::from_be_bytes([payload[pos + 3], payload[pos + 4]]) as usize;
                        let start = pos + 5;
                        if start + name_len <= end_ext {
                            return std::str::from_utf8(&payload[start..start + name_len])
                                .ok()
                                .map(|s| s.to_string());
                        }
                    }
                }
            }
            pos += ext_len;
        }

        None
    }

    pub fn sniff_http_host(payload: &[u8]) -> Option<String> {
        let text = std::str::from_utf8(payload).ok()?;
        for line in text.lines() {
            let lower = line.to_lowercase();
            if lower.starts_with("host:") {
                let host = line[5..].trim();
                let domain = host.split(':').next()?.trim();
                if !domain.is_empty() {
                    return Some(domain.to_string());
                }
            }
        }
        None
    }

    pub fn sniff_quic_sni(payload: &[u8]) -> Option<String> {
        // Basic Initial Packet SNI extraction
        if payload.len() > 100 && (payload[0] & 0x80 != 0) {
            return Self::sniff_tls_sni(&payload[40..]);
        }
        None
    }
}
