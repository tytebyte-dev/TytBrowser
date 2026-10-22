// Module: common\serial\serial.rs
// 1:1 Rust implementation corresponding to Go common\serial\serial.go

use crate::common::errors::Result;
use std::io::{Read, Write};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// ReadUint16 reads first two bytes from the reader, and then converts them to an uint16 value.
pub fn read_uint16<R: Read>(reader: &mut R) -> std::io::Result<u16> {
    let mut b = [0u8; 2];
    reader.read_exact(&mut b)?;
    Ok(u16::from_be_bytes(b))
}

/// WriteUint16 writes an uint16 value into writer.
pub fn write_uint16<W: Write>(writer: &mut W, value: u16) -> std::io::Result<usize> {
    let b = value.to_be_bytes();
    writer.write_all(&b)?;
    Ok(2)
}

/// WriteUint64 writes an uint64 value into writer.
pub fn write_uint64<W: Write>(writer: &mut W, value: u64) -> std::io::Result<usize> {
    let b = value.to_be_bytes();
    writer.write_all(&b)?;
    Ok(8)
}

pub async fn read_u16<R: AsyncRead + Unpin>(reader: &mut R) -> Result<u16> {
    Ok(reader.read_u16().await?)
}

pub async fn write_u16<W: AsyncWrite + Unpin>(writer: &mut W, val: u16) -> Result<()> {
    writer.write_u16(val).await?;
    Ok(())
}

pub async fn read_u32<R: AsyncRead + Unpin>(reader: &mut R) -> Result<u32> {
    Ok(reader.read_u32().await?)
}

pub async fn write_u32<W: AsyncWrite + Unpin>(writer: &mut W, val: u32) -> Result<()> {
    writer.write_u32(val).await?;
    Ok(())
}

pub async fn read_u64<R: AsyncRead + Unpin>(reader: &mut R) -> Result<u64> {
    Ok(reader.read_u64().await?)
}

pub async fn write_u64<W: AsyncWrite + Unpin>(writer: &mut W, val: u64) -> Result<()> {
    writer.write_u64(val).await?;
    Ok(())
}
