// Module: transport\internet\udp\mod.rs
// 1:1 Rust implementation corresponding to Go transport\internet\udp

pub mod config;
#[path = "config.pb.rs"]
pub mod config_pb;
pub mod dialer;
pub mod dispatcher;
pub mod hub;
pub mod hub_darwin;
pub mod hub_freebsd;
pub mod hub_linux;
pub mod hub_other;
pub mod udp;

#[cfg(test)]
pub mod dispatcher_test;

pub use config::UdpConfig;
pub use dialer::UdpDialer;
pub use dispatcher::{
    ConnEntry, Dispatcher, DispatcherConn, LinkDispatcher, ResponseCallback, UdpDispatcher,
    dial_dispatcher,
};
pub use hub::{Hub, HubOption, UdpHub, UdpPacket};
pub use udp::PROTOCOL_NAME;
