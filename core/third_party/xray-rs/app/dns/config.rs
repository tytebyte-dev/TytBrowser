use std::net::IpAddr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QueryStrategy {
    #[default]
    UseIP,
    UseIPv4,
    UseIPv6,
}

impl QueryStrategy {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "use_ip4" | "useip4" | "useipv4" | "use_ipv4" => QueryStrategy::UseIPv4,
            "use_ip6" | "useip6" | "useipv6" | "use_ipv6" => QueryStrategy::UseIPv6,
            _ => QueryStrategy::UseIP,
        }
    }
}

pub fn is_local_tld_or_dotless(domain: &str) -> bool {
    if !domain.contains('.') {
        return true;
    }
    let lower = domain.to_lowercase();
    let local_tlds = [
        ".local",
        ".localdomain",
        ".localhost",
        ".lan",
        ".home.arpa",
        ".example",
        ".invalid",
        ".test",
    ];
    local_tlds.iter().any(|tld| lower.ends_with(tld))
}

#[derive(Debug, Clone, Default)]
pub struct DnsConfig {
    pub query_strategy: QueryStrategy,
    pub client_ip: Option<IpAddr>,
    pub disable_fallback: bool,
    pub disable_fallback_if_match: bool,
    pub servers: Vec<String>,
}
