pub mod browser_dialer;
pub mod config;
#[path = "config.pb.rs"]
pub mod config_pb;
pub mod dialer;
pub mod filelocker;
pub mod finalmask;
pub mod grpc;
pub mod happy_eyeballs;
pub mod header;
pub mod headers;
pub mod httpupgrade;
pub mod hysteria;
pub mod internet;
pub mod kcp;
pub mod memory_settings;
pub mod reality;
pub mod sockopt;
pub mod sockopt_freebsd;
pub mod sockopt_other;
pub mod splithttp;
pub mod stat;
pub mod system_dialer;
pub mod system_listener;
pub mod tagged;
pub mod tcp;
pub mod tcp_hub;
pub mod tls;
pub mod udp;
pub mod websocket;

#[cfg(test)]
pub mod dialer_test;
#[cfg(test)]
pub mod sockopt_linux_test;
#[cfg(test)]
pub mod sockopt_test;
#[cfg(test)]
pub mod system_listener_test;

pub use finalmask::{
    FragmentConfig, Fragmenter, NoiseConfig, NoiseGenerator, SalamanderObfuscator,
};
pub use grpc::GrpcStream;
pub use happy_eyeballs::{HappyEyeballsConfig, sort_ips};
pub use httpupgrade::HttpUpgradeStream;
pub use kcp::{AckSegment, DataSegment, KcpConnection};
pub use memory_settings::{MemorySettings, MemoryStreamConfig};
pub use reality::{RealityClient, RealityConfig, RealityServer};
pub use sockopt::{SocketConfig, SocketOptions};
pub use splithttp::{SplitHttpClient, SplitHttpConfig};
pub use stat::StatStream;
pub use tagged::TaggedDialer;
pub use tcp::{TcpDialer, TcpHub};
pub use tls::{TlsClient, TlsServer, generate_cert_hash, generate_cert_hash_hex};
pub use udp::{UdpHub, UdpPacket};
pub use websocket::WebSocketStream;

#[derive(Debug, Clone, Default)]
pub struct StreamSettings {
    pub network: String,
    pub security: String,
    pub reality_settings: Option<RealityConfig>,
    pub splithttp_settings: Option<SplitHttpConfig>,
}
