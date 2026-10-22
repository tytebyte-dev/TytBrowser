use crate::app::router::geo::GeoDatabase;
use crate::common::net::{Address, Network};
use crate::common::protocol::SessionContext;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

lazy_static::lazy_static! {
    static ref GLOBAL_GEO_DB: GeoDatabase = GeoDatabase::new();
}

#[derive(Debug, Clone)]
pub enum DomainMatcher {
    Plain(String),
    Exact(String),
    Suffix(String),
    Keyword(String),
    GeoSite(String),
}

impl DomainMatcher {
    pub fn parse(s: &str) -> Self {
        let s_lower = s.to_lowercase();
        if let Some(rest) = s_lower.strip_prefix("geosite:") {
            DomainMatcher::GeoSite(rest.to_string())
        } else if let Some(rest) = s_lower.strip_prefix("full:") {
            DomainMatcher::Exact(rest.to_string())
        } else if let Some(rest) = s_lower.strip_prefix("domain:") {
            DomainMatcher::Suffix(rest.to_string())
        } else if let Some(rest) = s_lower.strip_prefix("keyword:") {
            DomainMatcher::Keyword(rest.to_string())
        } else {
            DomainMatcher::Plain(s_lower)
        }
    }

    pub fn matches(&self, domain: &str) -> bool {
        let d = domain.to_lowercase();
        match self {
            DomainMatcher::Exact(pattern) => d == *pattern,
            DomainMatcher::Suffix(pattern) => {
                d == *pattern || d.ends_with(&format!(".{}", pattern))
            }
            DomainMatcher::Keyword(pattern) => d.contains(pattern),
            DomainMatcher::Plain(pattern) => {
                d == *pattern || d.ends_with(&format!(".{}", pattern)) || d.contains(pattern)
            }
            DomainMatcher::GeoSite(group) => GLOBAL_GEO_DB.match_geosite(group, &d),
        }
    }
}

#[derive(Debug, Clone)]
pub enum IpMatcher {
    Exact(IpAddr),
    CidrV4(Ipv4Addr, u8),
    CidrV6(Ipv6Addr, u8),
    Private,
    GeoIp(String),
}

impl IpMatcher {
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        let s_lower = s.to_lowercase();

        if let Some(rest) = s_lower.strip_prefix("geoip:") {
            if rest == "private" {
                return Some(IpMatcher::Private);
            }
            return Some(IpMatcher::GeoIp(rest.to_string()));
        }

        if s_lower == "private" {
            return Some(IpMatcher::Private);
        }

        // Check for CIDR
        if let Some((ip_part, mask_part)) = s.split_once('/')
            && let Ok(mask) = mask_part.parse::<u8>()
        {
            if let Ok(v4) = Ipv4Addr::from_str(ip_part) {
                return Some(IpMatcher::CidrV4(v4, mask));
            }
            if let Ok(v6) = Ipv6Addr::from_str(ip_part) {
                return Some(IpMatcher::CidrV6(v6, mask));
            }
        }

        if let Ok(ip) = IpAddr::from_str(s) {
            return Some(IpMatcher::Exact(ip));
        }

        None
    }

    pub fn matches(&self, ip: &IpAddr) -> bool {
        match self {
            IpMatcher::Exact(target_ip) => ip == target_ip,
            IpMatcher::Private => match ip {
                IpAddr::V4(v4) => v4.is_loopback() || v4.is_private() || v4.is_link_local(),
                IpAddr::V6(v6) => v6.is_loopback() || v6.is_unicast_link_local(),
            },
            IpMatcher::CidrV4(net, mask) => {
                if let IpAddr::V4(v4) = ip {
                    if *mask == 0 {
                        return true;
                    }
                    if *mask > 32 {
                        return false;
                    }
                    let net_u32 = u32::from_be_bytes(net.octets());
                    let ip_u32 = u32::from_be_bytes(v4.octets());
                    let mask_u32 = !((1u32 << (32 - mask)) - 1);
                    (ip_u32 & mask_u32) == (net_u32 & mask_u32)
                } else {
                    false
                }
            }
            IpMatcher::CidrV6(net, mask) => {
                if let IpAddr::V6(v6) = ip {
                    if *mask == 0 {
                        return true;
                    }
                    if *mask > 128 {
                        return false;
                    }
                    let net_u128 = u128::from_be_bytes(net.octets());
                    let ip_u128 = u128::from_be_bytes(v6.octets());
                    let mask_u128 = !((1u128 << (128 - mask)) - 1);
                    (ip_u128 & mask_u128) == (net_u128 & mask_u128)
                } else {
                    false
                }
            }
            IpMatcher::GeoIp(group) => GLOBAL_GEO_DB.match_geoip(group, ip),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub tag: String,
    pub balancer_tag: Option<String>,
    pub inbound_tags: Vec<String>,
    pub domain_matchers: Vec<DomainMatcher>,
    pub ip_matchers: Vec<IpMatcher>,
    pub source_ip_matchers: Vec<IpMatcher>,
    pub ports: Vec<u16>,
    pub port_ranges: Vec<(u16, u16)>,
    pub source_ports: Vec<u16>,
    pub network: Option<Network>,
    pub protocols: Vec<String>,
    pub user_emails: Vec<String>,
    pub process: Vec<String>,
}

impl Rule {
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            balancer_tag: None,
            inbound_tags: Vec::new(),
            domain_matchers: Vec::new(),
            ip_matchers: Vec::new(),
            source_ip_matchers: Vec::new(),
            ports: Vec::new(),
            port_ranges: Vec::new(),
            source_ports: Vec::new(),
            network: None,
            protocols: Vec::new(),
            user_emails: Vec::new(),
            process: Vec::new(),
        }
    }

    pub fn with_balancer(balancer_tag: impl Into<String>) -> Self {
        Self {
            tag: String::new(),
            balancer_tag: Some(balancer_tag.into()),
            inbound_tags: Vec::new(),
            domain_matchers: Vec::new(),
            ip_matchers: Vec::new(),
            source_ip_matchers: Vec::new(),
            ports: Vec::new(),
            port_ranges: Vec::new(),
            source_ports: Vec::new(),
            network: None,
            protocols: Vec::new(),
            user_emails: Vec::new(),
            process: Vec::new(),
        }
    }

    pub fn matches_destination_ip(&self, ip: &IpAddr) -> bool {
        if self.ip_matchers.is_empty() {
            return false;
        }
        for matcher in &self.ip_matchers {
            if matcher.matches(ip) {
                return true;
            }
        }
        false
    }

    pub fn matches_process(&self, src: Option<std::net::SocketAddr>, is_tcp: bool) -> bool {
        if self.process.is_empty() {
            return true;
        }
        let src = match src {
            Some(s) => s,
            None => return false,
        };

        let proc_info = match crate::common::net::ProcessFinder::find_process_by_socket(src, is_tcp)
        {
            Ok(Some(info)) => info,
            _ => return false,
        };

        let cur_pid = std::process::id();
        for p in &self.process {
            let p_norm = p.replace('\\', "/");
            if p_norm == "self/" {
                if proc_info.pid == cur_pid {
                    return true;
                }
                continue;
            }
            if p_norm == "xray/" {
                if proc_info.pid == cur_pid || proc_info.name.eq_ignore_ascii_case("xray") {
                    return true;
                }
                continue;
            }
            if p_norm.ends_with('/') {
                if proc_info
                    .path
                    .to_ascii_lowercase()
                    .starts_with(&p_norm.to_ascii_lowercase())
                {
                    return true;
                }
                continue;
            }
            if p_norm.contains('/') {
                if proc_info.path.eq_ignore_ascii_case(&p_norm) {
                    return true;
                }
                continue;
            }
            let clean_name = p_norm.trim_end_matches(".exe");
            if proc_info.name.eq_ignore_ascii_case(clean_name) {
                return true;
            }
        }
        false
    }

    pub fn matches(&self, session: &SessionContext) -> bool {
        if !self.process.is_empty() {
            let is_tcp = session.destination.network == Network::Tcp;
            if !self.matches_process(session.source, is_tcp) {
                return false;
            }
        }

        if !self.inbound_tags.is_empty()
            && !self
                .inbound_tags
                .iter()
                .any(|tag| tag == &session.inbound_tag)
        {
            return false;
        }

        if let Some(net) = self.network
            && net != session.destination.network
        {
            return false;
        }

        // Port matching: exact ports or port ranges
        let target_port = session.destination.port;
        if !self.ports.is_empty() || !self.port_ranges.is_empty() {
            let port_match = self.ports.contains(&target_port)
                || self
                    .port_ranges
                    .iter()
                    .any(|&(s, e)| target_port >= s && target_port <= e);
            if !port_match {
                return false;
            }
        }

        // Source port matching
        if !self.source_ports.is_empty() {
            let src_port = session.source.map(|s| s.port()).unwrap_or(0);
            if !self.source_ports.contains(&src_port) {
                return false;
            }
        }

        // Source IP matching
        if !self.source_ip_matchers.is_empty() {
            let src_ip = session.source.map(|s| s.ip());
            let matched = match src_ip {
                Some(ip) => self.source_ip_matchers.iter().any(|m| m.matches(&ip)),
                None => false,
            };
            if !matched {
                return false;
            }
        }

        // Protocol matching (e.g. "http", "tls", "bittorrent")
        if !self.protocols.is_empty() {
            let matched = match session.sniffed_protocol.as_deref() {
                Some(proto) => self.protocols.iter().any(|p| p.eq_ignore_ascii_case(proto)),
                None => false,
            };
            if !matched {
                return false;
            }
        }

        // User email matching
        if !self.user_emails.is_empty() {
            let user_email = session.user.as_ref().map(|u| u.email.as_str());
            let matched = match user_email {
                Some(email) => self.user_emails.iter().any(|e| e == email),
                None => false,
            };
            if !matched {
                return false;
            }
        }

        let has_domain_rules = !self.domain_matchers.is_empty();
        let has_ip_rules = !self.ip_matchers.is_empty();

        if !has_domain_rules && !has_ip_rules {
            return true;
        }

        // Check sniffed domain first if present
        if has_domain_rules && let Some(domain) = session.sniffed_domain.as_deref() {
            for matcher in &self.domain_matchers {
                if matcher.matches(domain) {
                    return true;
                }
            }
        }

        match &session.destination.address {
            Address::Domain(domain) => {
                if has_domain_rules {
                    for matcher in &self.domain_matchers {
                        if matcher.matches(domain) {
                            return true;
                        }
                    }
                }
                false
            }
            Address::Ipv4(v4) => {
                if has_ip_rules {
                    let ip = IpAddr::V4(*v4);
                    for matcher in &self.ip_matchers {
                        if matcher.matches(&ip) {
                            return true;
                        }
                    }
                }
                false
            }
            Address::Ipv6(v6) => {
                if has_ip_rules {
                    let ip = IpAddr::V6(*v6);
                    for matcher in &self.ip_matchers {
                        if matcher.matches(&ip) {
                            return true;
                        }
                    }
                }
                false
            }
        }
    }
}
