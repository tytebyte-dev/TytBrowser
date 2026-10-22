pub mod config;
pub mod tcp;
pub mod udp;

pub use config::{CustomHeaderConfig, TCPConfig, TCPItem, TCPSequence, UDPConfig, UDPItem};
pub use tcp::{TCP_CUSTOM_MAGIC, TcpCustomConn, client_handshake, server_handshake};
pub use udp::{UDP_CUSTOM_MAGIC, UdpCustomClient, UdpCustomServer};
