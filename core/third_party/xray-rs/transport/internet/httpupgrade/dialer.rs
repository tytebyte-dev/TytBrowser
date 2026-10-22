// Module: transport\internet\httpupgrade\dialer.rs
// 1:1 Rust implementation corresponding to Go transport\internet\httpupgrade\dialer.go

use super::config::HttpUpgradeConfig;
use super::httpupgrade::HttpUpgradeStream;
use crate::common::errors::Result;
use crate::common::net::{BoxStream, Destination};
use crate::transport::internet::tcp::TcpDialer;

pub struct HttpUpgradeDialer;

impl HttpUpgradeDialer {
    pub async fn dial(dest: &Destination, config: &HttpUpgradeConfig) -> Result<BoxStream> {
        let tcp_stream = TcpDialer::dial(dest).await?;
        let host = if config.host.is_empty() {
            dest.address.to_string()
        } else {
            config.host.clone()
        };
        let path = config.get_normalized_path();
        HttpUpgradeStream::client_handshake(&host, &path, tcp_stream, &config.headers).await
    }
}
