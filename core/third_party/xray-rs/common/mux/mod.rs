pub mod client;
pub mod frame;
pub mod mux;
pub mod reader;
pub mod server;
pub mod session;
pub mod writer;

#[cfg(test)]
pub mod client_test;
#[cfg(test)]
pub mod frame_test;
#[cfg(test)]
pub mod mux_test;
#[cfg(test)]
pub mod server_test;
#[cfg(test)]
pub mod session_test;

pub use client::{Client, MuxClient};
pub use frame::{
    Frame, FrameMetadata, FrameType, OPTION_DATA, OPTION_ERROR, SessionStatus, TARGET_NETWORK_TCP,
    TARGET_NETWORK_UDP,
};
pub use mux::ClientStrategy;
pub use reader::{FrameReader, PacketReader, StreamReader};
pub use server::{MuxServer, Server};
pub use session::{Session, SessionManager};
pub use writer::{FrameWriter, Writer};
