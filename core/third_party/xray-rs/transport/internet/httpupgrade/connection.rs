// Module: transport\internet\httpupgrade\connection.rs
// 1:1 Rust implementation corresponding to Go transport\internet\httpupgrade\connection.go

use crate::common::net::BoxStream;

pub struct HttpUpgradeConnection {
    pub stream: BoxStream,
    pub remote_addr: String,
}

impl HttpUpgradeConnection {
    pub fn new(stream: BoxStream, remote_addr: impl Into<String>) -> Self {
        Self {
            stream,
            remote_addr: remote_addr.into(),
        }
    }

    pub fn remote_addr(&self) -> &str {
        &self.remote_addr
    }

    pub fn into_inner(self) -> BoxStream {
        self.stream
    }
}
