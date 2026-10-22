use crate::common::errors::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Port(pub u16);

impl Port {
    pub fn new(val: u16) -> Self {
        Port(val)
    }

    pub fn value(&self) -> u16 {
        self.0
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 2 {
            return Err(Error::Protocol("Port bytes must have length >= 2".into()));
        }
        Ok(Port(u16::from_be_bytes([bytes[0], bytes[1]])))
    }
}

impl fmt::Display for Port {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Port {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        s.parse::<u16>()
            .map(Port)
            .map_err(|e| Error::Protocol(format!("Invalid port string '{}': {}", s, e)))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortRange {
    pub from: u16,
    pub to: u16,
}

impl PortRange {
    pub fn new(from: u16, to: u16) -> Self {
        Self { from, to }
    }

    pub fn single(port: u16) -> Self {
        Self {
            from: port,
            to: port,
        }
    }

    pub fn contains(&self, port: Port) -> bool {
        self.from <= port.0 && port.0 <= self.to
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortList {
    pub ranges: Vec<PortRange>,
}

impl PortList {
    pub fn new() -> Self {
        Self { ranges: Vec::new() }
    }

    pub fn add_range(&mut self, from: u16, to: u16) {
        self.ranges.push(PortRange::new(from, to));
    }

    pub fn add_single(&mut self, port: u16) {
        self.ranges.push(PortRange::single(port));
    }

    pub fn contains(&self, port: Port) -> bool {
        self.ranges.iter().any(|r| r.contains(port))
    }
}
