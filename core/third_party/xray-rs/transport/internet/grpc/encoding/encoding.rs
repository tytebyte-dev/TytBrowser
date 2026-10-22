// Module: transport\internet\grpc\encoding\encoding.rs
// 1:1 Rust implementation corresponding to Go transport\internet\grpc\encoding\encoding.go

use crate::common::errors::{Error, Result};

pub fn encode_grpc_frame(payload: &[u8]) -> Vec<u8> {
    let mut frame = Vec::with_capacity(5 + payload.len());
    frame.push(0); // uncompressed
    frame.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    frame.extend_from_slice(payload);
    frame
}

pub fn decode_grpc_frame(data: &[u8]) -> Result<(&[u8], &[u8])> {
    if data.len() < 5 {
        return Err(Error::Protocol("Incomplete gRPC frame header".into()));
    }
    let len = u32::from_be_bytes([data[1], data[2], data[3], data[4]]) as usize;
    if data.len() < 5 + len {
        return Err(Error::Protocol("Incomplete gRPC frame payload".into()));
    }
    Ok((&data[5..5 + len], &data[5 + len..]))
}
