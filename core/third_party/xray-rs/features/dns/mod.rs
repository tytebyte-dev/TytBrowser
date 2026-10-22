// Module: features\dns\mod.rs

pub mod client;
pub mod fakedns;
pub mod localdns;

pub use client::{DEFAULT_TTL, DnsClient, IPOption, RCodeError, client_type};
pub use fakedns::{FAKE_IPV4_POOL, FAKE_IPV6_POOL, FakeDnsEngine, FakeDnsFeature, fake_dns_type};
pub use localdns::LocalDnsClient;
