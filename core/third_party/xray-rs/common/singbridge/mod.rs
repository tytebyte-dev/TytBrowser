pub mod destination;
pub mod dialer;
pub mod error;
pub mod handler;
pub mod logger;
pub mod packet;
pub mod pipe;
pub mod reader;

#[cfg(test)]
pub mod singbridge_test;

pub use destination::{Socksaddr, from_destination, to_destination, to_network, to_socksaddr};
pub use dialer::{
    SingDialer, SingDialerTrait, XrayDialer, XrayOutboundDialer, dial_singbox_bridge,
};
pub use error::{is_closed_or_canceled, return_error, return_result, wrap_sing_error};
pub use handler::{Dispatcher, Metadata, SingHandler, TcpConnectionHandler, UdpConnectionHandler};
pub use logger::{ContextLogger, SingLogger, XrayLogger};
pub use packet::{PacketConnWrapper, SingPacket, copy_packet_conn};
pub use pipe::{PipeConnWrapper, bridge_pipe, copy_conn};
pub use reader::{Conn, SingReader};
