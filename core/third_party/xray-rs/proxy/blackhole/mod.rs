pub mod blackhole;
pub mod config;
#[path = "config.pb.rs"]
pub mod config_pb;

#[cfg(test)]
pub mod blackhole_test;
#[cfg(test)]
pub mod config_test;

use crate::common::errors::Result;
use crate::common::net::BoxStream;
use crate::common::protocol::SessionContext;
use crate::features::outbound::OutboundHandler;
use async_trait::async_trait;

pub use blackhole::BlackholeHandler;
pub use config::{BlackholeConfig, ResponseType};

pub struct Handler {
    tag: String,
    response: ResponseType,
}

impl Handler {
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            response: ResponseType::None,
        }
    }

    pub fn with_response(tag: impl Into<String>, response: ResponseType) -> Self {
        Self {
            tag: tag.into(),
            response,
        }
    }
}

#[async_trait]
impl OutboundHandler for Handler {
    fn tag(&self) -> &str {
        &self.tag
    }

    async fn connect(&self, _session: &SessionContext) -> Result<BoxStream> {
        let (mut client_stream, server_stream) = tokio::io::duplex(1024);
        match self.response {
            ResponseType::Http403 => {
                tokio::spawn(async move {
                    use tokio::io::AsyncWriteExt;
                    let _ = client_stream
                        .write_all(b"HTTP/1.1 403 Forbidden\r\nConnection: close\r\n\r\n")
                        .await;
                    let _ = client_stream.flush().await;
                });
                Ok(Box::pin(server_stream))
            }
            ResponseType::Http500 => {
                tokio::spawn(async move {
                    use tokio::io::AsyncWriteExt;
                    let _ = client_stream
                        .write_all(
                            b"HTTP/1.1 500 Internal Server Error\r\nConnection: close\r\n\r\n",
                        )
                        .await;
                    let _ = client_stream.flush().await;
                });
                Ok(Box::pin(server_stream))
            }
            ResponseType::None => {
                drop(client_stream);
                Ok(Box::pin(server_stream))
            }
        }
    }
}
