use crate::common::net::Destination;
use crate::common::protocol::user::{MemoryUser, SecurityType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestCommand {
    Tcp = 0x01,
    Udp = 0x02,
    Mux = 0x03,
    Rvs = 0x04,
}

impl RequestCommand {
    pub fn to_u8(&self) -> u8 {
        *self as u8
    }

    pub fn from_u8(b: u8) -> Option<Self> {
        match b {
            0x01 => Some(RequestCommand::Tcp),
            0x02 => Some(RequestCommand::Udp),
            0x03 => Some(RequestCommand::Mux),
            0x04 => Some(RequestCommand::Rvs),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RequestHeader {
    pub version: u8,
    pub command: RequestCommand,
    pub option: u8,
    pub security: SecurityType,
    pub destination: Destination,
    pub user: Option<MemoryUser>,
}

impl RequestHeader {
    pub fn new(command: RequestCommand, destination: Destination) -> Self {
        Self {
            version: 1,
            command,
            option: 0,
            security: SecurityType::Auto,
            destination,
            user: None,
        }
    }
}
