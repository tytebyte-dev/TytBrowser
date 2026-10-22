pub mod address;
pub mod cnc;
pub mod destination;
pub mod find_process;
pub mod find_process_linux;
pub mod find_process_others;
pub mod find_process_windows;
pub mod net;
pub mod network;
pub mod port;
pub mod system;

#[cfg(test)]
pub mod address_test;
#[cfg(test)]
pub mod destination_test;
#[cfg(test)]
pub mod find_process_test;
#[cfg(test)]
pub mod port_test;

use std::pin::Pin;
use tokio::io::{AsyncRead, AsyncWrite};

pub use address::{Address, domain_address, ip_address, parse_address};
pub use cnc::CncConnection;
pub use destination::{Destination, parse_destination, tcp_destination, udp_destination};
pub use find_process::ProcessFinder;
pub use network::{Network, NetworkList};
pub use port::{Port, PortList, PortRange};
pub use system::SystemListener;

/// A pinned, boxed trait object for bidirectional async byte streams.
pub type BoxStream = Pin<Box<dyn AsyncStream>>;

/// Trait alias combining AsyncRead, AsyncWrite, Send, Sync, Unpin and 'static.
pub trait AsyncStream: AsyncRead + AsyncWrite + Send + Sync + Unpin + 'static {}

impl<T> AsyncStream for T where T: AsyncRead + AsyncWrite + Send + Sync + Unpin + 'static {}
