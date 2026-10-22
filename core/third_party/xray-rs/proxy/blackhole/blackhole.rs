// Module: proxy\blackhole\blackhole.rs
// 1:1 Rust implementation corresponding to Go proxy\blackhole\blackhole.go

use super::config::{BlackholeConfig, ResponseType};
use crate::common::errors::Result;
use tokio::io::{AsyncWrite, AsyncWriteExt};

pub struct BlackholeHandler {
    config: BlackholeConfig,
}

impl BlackholeHandler {
    pub fn new(config: BlackholeConfig) -> Self {
        Self { config }
    }

    pub async fn handle<W: AsyncWrite + Unpin>(&self, mut writer: W) -> Result<()> {
        match self.config.response {
            ResponseType::None => Ok(()),
            ResponseType::Http403 => {
                writer
                    .write_all(b"HTTP/1.1 403 Forbidden\r\nConnection: close\r\n\r\n")
                    .await?;
                writer.flush().await?;
                Ok(())
            }
            ResponseType::Http500 => {
                writer
                    .write_all(b"HTTP/1.1 500 Internal Server Error\r\nConnection: close\r\n\r\n")
                    .await?;
                writer.flush().await?;
                Ok(())
            }
        }
    }
}
