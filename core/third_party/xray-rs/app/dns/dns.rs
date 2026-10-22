// Module: app\dns\dns.rs
// 1:1 Rust implementation corresponding to Go app\dns\dns.go

use async_trait::async_trait;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use crate::app::dns::cache_controller::CacheController;
use crate::app::dns::config::QueryStrategy;
use crate::app::dns::hosts::StaticHosts;
use crate::app::dns::nameserver::NameServer;
use crate::app::dns::nameserver_doh::DohNameServer;
use crate::app::dns::nameserver_local::LocalNameServer;
use crate::app::dns::nameserver_tcp::TcpNameServer;
use crate::app::dns::nameserver_udp::UdpNameServer;
use crate::common::errors::{Error, Result};
use crate::features::dns::client::{DnsClient as DnsClientTrait, IPOption};
use crate::features::feature::{Feature, TYPE_DNS_CLIENT};

#[derive(Debug, Clone)]
pub struct DnsServerConfig {
    pub address: String,
    pub domains: Vec<String>,
    pub skip_fallback: bool,
    pub tag: Option<String>,
}

pub struct DnsClient {
    hosts: Arc<StaticHosts>,
    cache: Arc<CacheController>,
    servers: Vec<DnsServerConfig>,
    nameservers: Vec<Arc<dyn NameServer>>,
    query_strategy: QueryStrategy,
}

impl Default for DnsClient {
    fn default() -> Self {
        Self::new()
    }
}

impl DnsClient {
    pub fn new() -> Self {
        Self {
            hosts: Arc::new(StaticHosts::new()),
            cache: Arc::new(CacheController::new(
                "default".into(),
                false,
                true,
                Duration::from_secs(3600),
            )),
            servers: Vec::new(),
            nameservers: Vec::new(),
            query_strategy: QueryStrategy::UseIP,
        }
    }

    pub fn with_hosts(hosts_map: HashMap<String, Vec<IpAddr>>) -> Self {
        let hosts = StaticHosts::new();
        for (domain, ips) in hosts_map {
            hosts.add_exact(&domain, ips);
        }

        Self {
            hosts: Arc::new(hosts),
            cache: Arc::new(CacheController::new(
                "default".into(),
                false,
                true,
                Duration::from_secs(3600),
            )),
            servers: Vec::new(),
            nameservers: Vec::new(),
            query_strategy: QueryStrategy::UseIP,
        }
    }

    pub fn set_query_strategy(&mut self, strategy: QueryStrategy) {
        self.query_strategy = strategy;
    }

    pub fn query_strategy(&self) -> QueryStrategy {
        self.query_strategy
    }

    pub fn add_host(&self, domain: &str, ips: Vec<IpAddr>) {
        self.hosts.add_exact(domain, ips);
    }

    pub fn add_nameserver(&mut self, ns: Arc<dyn NameServer>) {
        self.nameservers.push(ns);
    }

    pub fn add_server(&mut self, server: DnsServerConfig) {
        let addr = server.address.trim();
        if addr.starts_with("https://") || addr.starts_with("http://") {
            self.nameservers.push(Arc::new(DohNameServer::new(addr)));
        } else if addr.eq_ignore_ascii_case("local") || addr.eq_ignore_ascii_case("localhost") {
            self.nameservers.push(Arc::new(LocalNameServer::new()));
        } else if let Some(stripped) = addr.strip_prefix("tcp://") {
            if let Ok(sa) = stripped.parse::<SocketAddr>() {
                self.nameservers.push(Arc::new(TcpNameServer::new(sa)));
            } else if let Ok(ip) = stripped.parse::<IpAddr>() {
                self.nameservers
                    .push(Arc::new(TcpNameServer::new(SocketAddr::new(ip, 53))));
            }
        } else if let Some(stripped) = addr.strip_prefix("udp://") {
            if let Ok(sa) = stripped.parse::<SocketAddr>() {
                self.nameservers.push(Arc::new(UdpNameServer::new(sa)));
            } else if let Ok(ip) = stripped.parse::<IpAddr>() {
                self.nameservers
                    .push(Arc::new(UdpNameServer::new(SocketAddr::new(ip, 53))));
            }
        } else if let Ok(sa) = addr.parse::<SocketAddr>() {
            self.nameservers.push(Arc::new(UdpNameServer::new(sa)));
        } else if let Ok(ip) = addr.parse::<IpAddr>() {
            self.nameservers
                .push(Arc::new(UdpNameServer::new(SocketAddr::new(ip, 53))));
        }

        self.servers.push(server);
    }

    fn filter_ips(&self, ips: &[IpAddr], option: &IPOption) -> Vec<IpAddr> {
        let (ipv4_allow, ipv6_allow) = match self.query_strategy {
            QueryStrategy::UseIPv4 => (true, false),
            QueryStrategy::UseIPv6 => (false, true),
            QueryStrategy::UseIP => (option.ipv4_enable, option.ipv6_enable),
        };

        ips.iter()
            .copied()
            .filter(|ip| match ip {
                IpAddr::V4(_) => ipv4_allow,
                IpAddr::V6(_) => ipv6_allow,
            })
            .collect()
    }

    pub async fn lookup_ip(&self, domain: &str) -> Result<Vec<IpAddr>> {
        self.lookup_ip_with_option(domain, IPOption::default())
            .await
            .map(|(ips, _)| ips)
    }

    pub async fn lookup_ip_with_option(
        &self,
        domain: &str,
        option: IPOption,
    ) -> Result<(Vec<IpAddr>, u32)> {
        let clean = domain.trim_end_matches('.');

        // 1. Static Hosts
        if let Some(ips) = self.hosts.lookup(clean) {
            let filtered = self.filter_ips(&ips, &option);
            if !filtered.is_empty() {
                return Ok((filtered, 300));
            }
        }

        // 2. Cache
        if let Some(ips) = self.cache.get(clean) {
            let filtered = self.filter_ips(&ips, &option);
            if !filtered.is_empty() {
                return Ok((filtered, 300));
            }
        }

        // 3. Upstream NameServers
        for ns in &self.nameservers {
            if let Ok(ips) = ns.query_ip(clean).await
                && !ips.is_empty()
            {
                self.cache.set(clean, ips.clone(), Duration::from_secs(300));
                let filtered = self.filter_ips(&ips, &option);
                if !filtered.is_empty() {
                    return Ok((filtered, 300));
                }
            }
        }

        // 4. Local resolution fallback
        if let Ok(ips) = LocalNameServer::new().query_ip(clean).await
            && !ips.is_empty()
        {
            self.cache.set(clean, ips.clone(), Duration::from_secs(300));
            let filtered = self.filter_ips(&ips, &option);
            if !filtered.is_empty() {
                return Ok((filtered, 300));
            }
        }

        Err(Error::NotFound(format!(
            "No IP found for domain: {}",
            domain
        )))
    }
}

impl Feature for DnsClient {
    fn feature_type(&self) -> &'static str {
        TYPE_DNS_CLIENT
    }

    fn start(&self) -> Result<()> {
        Ok(())
    }

    fn close(&self) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl DnsClientTrait for DnsClient {
    async fn lookup_ip_with_option(
        &self,
        domain: &str,
        option: IPOption,
    ) -> Result<(Vec<IpAddr>, u32)> {
        DnsClient::lookup_ip_with_option(self, domain, option).await
    }
}
