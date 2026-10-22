// Module: transport\internet\header.rs
// 1:1 Rust implementation corresponding to Go transport\internet\header.go

use crate::common::net::BoxStream;

pub trait PacketHeader: Send + Sync {
    fn size(&self) -> i32;
    fn serialize(&self, buffer: &mut [u8]);
}

pub trait ConnectionAuthenticator: Send + Sync {
    fn client(&self, stream: BoxStream) -> BoxStream;
    fn server(&self, stream: BoxStream) -> BoxStream;
}

pub trait Header: Send + Sync {
    fn size(&self) -> usize;
    fn serialize(&self, buffer: &mut [u8]) -> usize;
}

pub struct NoneHeader;

impl Header for NoneHeader {
    fn size(&self) -> usize {
        0
    }
    fn serialize(&self, _buffer: &mut [u8]) -> usize {
        0
    }
}

impl PacketHeader for NoneHeader {
    fn size(&self) -> i32 {
        0
    }
    fn serialize(&self, _buffer: &mut [u8]) {}
}
