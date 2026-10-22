pub mod config;
pub mod dial;
pub mod encoding;
pub mod grpc;
pub mod hub;

#[cfg(test)]
pub mod config_test;

pub use config::GrpcConfig;
pub use dial::GrpcDialer;
pub use grpc::{DEFAULT_SERVICE_NAME, PROTOCOL_NAME};
pub use hub::GrpcListener;

use crate::common::errors::{Error, Result};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub struct GrpcStream;

impl GrpcStream {
    pub async fn encode_frame<W: AsyncWrite + Unpin>(writer: &mut W, data: &[u8]) -> Result<()> {
        writer.write_u8(0).await?;
        writer.write_u32(data.len() as u32).await?;
        writer.write_all(data).await?;
        writer.flush().await?;
        Ok(())
    }

    pub async fn decode_frame<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Vec<u8>> {
        let _flag = reader.read_u8().await?;
        let len = reader.read_u32().await? as usize;
        if len > 16 * 1024 * 1024 {
            return Err(Error::Protocol("gRPC frame too large".into()));
        }
        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf).await?;
        Ok(buf)
    }
}
