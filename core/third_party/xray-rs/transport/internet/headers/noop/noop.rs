// Module: transport\internet\headers\noop\noop.rs
// 1:1 Rust implementation corresponding to Go transport\internet\headers\noop\noop.go

use crate::common::net::BoxStream;
use crate::transport::internet::header::{ConnectionAuthenticator, Header, PacketHeader};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NoOpHeader;

impl NoOpHeader {
    pub fn new() -> Self {
        Self
    }
}

impl Header for NoOpHeader {
    fn size(&self) -> usize {
        0
    }
    fn serialize(&self, _buffer: &mut [u8]) -> usize {
        0
    }
}

impl PacketHeader for NoOpHeader {
    fn size(&self) -> i32 {
        0
    }
    fn serialize(&self, _buffer: &mut [u8]) {}
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NoOpConnectionHeader;

impl NoOpConnectionHeader {
    pub fn new() -> Self {
        Self
    }
}

impl ConnectionAuthenticator for NoOpConnectionHeader {
    fn client(&self, stream: BoxStream) -> BoxStream {
        stream
    }

    fn server(&self, stream: BoxStream) -> BoxStream {
        stream
    }
}
