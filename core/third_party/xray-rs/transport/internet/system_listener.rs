// Module: transport\internet\system_listener.rs
// 1:1 Rust implementation corresponding to Go transport\internet\system_listener.go

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::RwLock;

use super::system_dialer::ControllerFunc;
use crate::common::errors::{Error, Result};
use crate::transport::internet::filelocker::FileLocker;
use crate::transport::internet::sockopt::SocketOptions;

pub struct UnixListenerWrapper {
    pub lock_path: Option<String>,
}

impl Drop for UnixListenerWrapper {
    fn drop(&mut self) {
        if let Some(ref path) = self.lock_path {
            let mut locker = FileLocker::new(path);
            locker.release();
        }
    }
}

pub struct SystemListener {
    listener: TcpListener,
    controllers: Arc<RwLock<Vec<ControllerFunc>>>,
}

impl SystemListener {
    pub async fn bind(addr: SocketAddr) -> Result<Self> {
        Self::listen_tcp(addr, None).await
    }

    pub async fn listen_tcp(addr: SocketAddr, _sockopt: Option<&SocketOptions>) -> Result<Self> {
        let listener = TcpListener::bind(addr).await.map_err(Error::Io)?;
        Ok(Self {
            listener,
            controllers: Arc::new(RwLock::new(Vec::new())),
        })
    }

    pub async fn listen_packet(
        addr: SocketAddr,
        _sockopt: Option<&SocketOptions>,
    ) -> Result<UdpSocket> {
        let socket = UdpSocket::bind(addr).await.map_err(Error::Io)?;
        Ok(socket)
    }

    pub async fn add_controller(&self, ctl: ControllerFunc) {
        let mut list = self.controllers.write().await;
        list.push(ctl);
    }

    pub fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.listener.local_addr()?)
    }

    pub async fn accept(&self) -> Result<(TcpStream, SocketAddr)> {
        let (stream, remote_addr) = self.listener.accept().await?;
        let _ = stream.set_nodelay(true);

        // Execute registered controllers
        {
            let ctls = self.controllers.read().await;
            for ctl in ctls.iter() {
                let _ = ctl("tcp", &remote_addr.to_string());
            }
        }

        Ok((stream, remote_addr))
    }
}
