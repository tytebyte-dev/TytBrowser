// Module: transport\internet\websocket\connection.rs
// 1:1 Rust implementation corresponding to Go transport\internet\websocket\connection.go

use crate::common::net::BoxStream;

pub struct WebSocketConnection {
    stream: BoxStream,
    remote_addr: String,
}

impl WebSocketConnection {
    pub fn new(stream: BoxStream, remote_addr: impl Into<String>) -> Self {
        Self {
            stream,
            remote_addr: remote_addr.into(),
        }
    }

    pub fn remote_addr(&self) -> &str {
        &self.remote_addr
    }

    pub fn into_stream(self) -> BoxStream {
        self.stream
    }
}
