// Module: transport\internet\tcp\hub.rs
// 1:1 Rust implementation corresponding to Go transport\internet\tcp\hub.go

use std::net::SocketAddr;
use tokio::net::TcpListener as TokioTcpListener;

use crate::common::errors::Result;
use crate::common::net::BoxStream;

pub struct TcpHub {
    listener: TokioTcpListener,
    local_addr: SocketAddr,
}

impl TcpHub {
    pub async fn listen(addr: SocketAddr) -> Result<Self> {
        let listener = TokioTcpListener::bind(addr).await?;
        let local_addr = listener.local_addr()?;
        Ok(Self {
            listener,
            local_addr,
        })
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    pub async fn accept(&self) -> Result<(BoxStream, SocketAddr)> {
        let (stream, remote_addr) = self.listener.accept().await?;
        let _ = stream.set_nodelay(true);
        Ok((Box::pin(stream), remote_addr))
    }
}
