// Module: common\protocol\dns\io.rs
// 1:1 Rust implementation corresponding to Go common\protocol\dns\io.go

use crate::common::buf::Buffer;
use crate::common::errors::{Error, Result};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub struct DnsTcpReader;

impl DnsTcpReader {
    pub async fn read_message<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Buffer> {
        let size = reader.read_u16().await.map_err(Error::Io)? as usize;
        let mut data = vec![0u8; size];
        reader.read_exact(&mut data).await.map_err(Error::Io)?;
        Ok(Buffer::from_bytes(&data))
    }
}

pub struct DnsTcpWriter;

impl DnsTcpWriter {
    pub async fn write_message<W: AsyncWrite + Unpin>(writer: &mut W, buf: &Buffer) -> Result<()> {
        let len = buf.len() as u16;
        writer.write_u16(len).await.map_err(Error::Io)?;
        writer.write_all(buf.as_slice()).await.map_err(Error::Io)?;
        Ok(())
    }
}
