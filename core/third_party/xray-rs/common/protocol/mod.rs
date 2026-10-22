pub mod account;
pub mod address;
pub mod bittorrent;
pub mod context;
pub mod dns;
pub mod headers;
pub mod http;
pub mod id;
pub mod payload;
pub mod protocol;
pub mod quic;
pub mod server_spec;
pub mod time;
pub mod tls;
pub mod udp;
pub mod user;

#[cfg(test)]
pub mod address_test;
#[cfg(test)]
pub mod id_test;
#[cfg(test)]
pub mod time_test;

use crate::common::net::Destination;
use std::net::SocketAddr;

pub use account::{Account, AsAccount};
pub use address::AddressParser;
pub use bittorrent::BittorrentSniffer;
pub use context::ProtocolContext;
pub use headers::{RequestCommand, RequestHeader};
pub use id::Id;
pub use payload::{AddressType, TransferType};
pub use server_spec::ServerSpec;
pub use time::Timestamp;
pub use udp::UdpPacket;
pub use user::{MemoryUser, SecurityType, User};

#[derive(Debug, Clone)]
pub struct SessionContext {
    pub inbound_tag: String,
    pub destination: Destination,
    pub source: Option<SocketAddr>,
    pub outbound_tag: Option<String>,
    pub user: Option<User>,
    pub sniffing_request: Option<crate::common::session::SniffingRequest>,
    pub route_target: Option<Destination>,
    pub original_destination: Option<Destination>,
    pub forced_outbound_tag: Option<String>,
    pub sniffed_protocol: Option<String>,
    pub sniffed_domain: Option<String>,
}

impl SessionContext {
    pub fn new(inbound_tag: impl Into<String>, destination: Destination) -> Self {
        Self {
            inbound_tag: inbound_tag.into(),
            original_destination: Some(destination.clone()),
            destination,
            source: None,
            outbound_tag: None,
            user: None,
            sniffing_request: None,
            route_target: None,
            forced_outbound_tag: None,
            sniffed_protocol: None,
            sniffed_domain: None,
        }
    }

    pub fn with_sniffing(mut self, req: crate::common::session::SniffingRequest) -> Self {
        self.sniffing_request = Some(req);
        self
    }

    pub fn with_forced_outbound(mut self, tag: impl Into<String>) -> Self {
        self.forced_outbound_tag = Some(tag.into());
        self
    }

    pub fn with_user(mut self, user: User) -> Self {
        self.user = Some(user);
        self
    }
}
