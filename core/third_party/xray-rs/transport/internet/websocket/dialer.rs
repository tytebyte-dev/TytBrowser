// Module: transport\internet\websocket\dialer.rs
// 1:1 Rust implementation corresponding to Go transport\internet\websocket\dialer.go

use super::config::WebSocketConfig;
use super::ws::WebSocketStream;
use crate::common::errors::Result;
use crate::common::net::{BoxStream, Destination};
use crate::transport::internet::tcp::TcpDialer;

pub struct WebSocketDialer;

impl WebSocketDialer {
    pub async fn dial(dest: &Destination, config: &WebSocketConfig) -> Result<BoxStream> {
        let tcp_stream = TcpDialer::dial(dest).await?;
        let host = dest.address.to_string();
        let path = if config.path.is_empty() {
            "/"
        } else {
            &config.path
        };
        let url = format!("ws://{}{}", host, path);
        WebSocketStream::client_handshake(&url, Some(&host), tcp_stream).await
    }
}
