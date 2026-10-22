pub mod cache_controller;
pub mod config;
pub mod dns;
pub mod dnscommon;
pub mod fakedns;
pub mod hosts;
pub mod nameserver;
pub mod nameserver_cached;
pub mod nameserver_doh;
pub mod nameserver_fakedns;
pub mod nameserver_local;
pub mod nameserver_quic;
pub mod nameserver_tcp;
pub mod nameserver_udp;

#[cfg(test)]
pub mod dns_test;
#[cfg(test)]
pub mod dnscommon_test;
#[cfg(test)]
pub mod hosts_test;
#[cfg(test)]
pub mod nameserver_doh_test;
#[cfg(test)]
pub mod nameserver_local_test;
#[cfg(test)]
pub mod nameserver_quic_test;
#[cfg(test)]
pub mod nameserver_tcp_test;

pub use cache_controller::{CacheController, DnsRecord};
pub use config::{DnsConfig, QueryStrategy, is_local_tld_or_dotless};
pub use dns::{DnsClient, DnsServerConfig};
pub use dnscommon::{IPRecord, fqdn};
pub use fakedns::FakeDnsHolder;
pub use hosts::StaticHosts;
pub use nameserver::NameServer;
pub use nameserver_cached::CachedNameServer;
pub use nameserver_doh::DohNameServer;
pub use nameserver_fakedns::FakeDnsNameServer;
pub use nameserver_local::LocalNameServer;
pub use nameserver_quic::QuicNameServer;
pub use nameserver_tcp::TcpNameServer;
pub use nameserver_udp::UdpNameServer;
