pub mod client;
pub mod config;
pub mod dns;
pub mod server;

#[cfg(test)]
pub mod dns_test;
#[cfg(test)]
pub mod xdns_test;

pub use client::XDnsClient;
pub use config::XDnsConfig;
pub use dns::*;
pub use server::XDnsServer;
