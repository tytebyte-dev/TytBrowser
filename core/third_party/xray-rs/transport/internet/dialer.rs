// Module: transport\internet\dialer.rs
// 1:1 Rust implementation corresponding to Go transport\internet\dialer.go

use async_trait::async_trait;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, OnceLock, RwLock};

use super::memory_settings::MemoryStreamConfig;
use super::sockopt::SocketOptions;
use super::system_dialer::SystemDialer;
use crate::common::errors::{Error, Result};
use crate::common::net::{BoxStream, Destination, Network};

#[async_trait]
pub trait Dialer: Send + Sync {
    async fn dial(&self, dest: &Destination) -> Result<BoxStream>;
}

pub type DialFuture = Pin<Box<dyn Future<Output = Result<BoxStream>> + Send>>;
pub type TransportDialerFn =
    Arc<dyn Fn(&Destination, Option<&MemoryStreamConfig>) -> DialFuture + Send + Sync>;

static TRANSPORT_DIALER_REGISTRY: OnceLock<RwLock<HashMap<String, TransportDialerFn>>> =
    OnceLock::new();

fn get_transport_dialer_registry() -> &'static RwLock<HashMap<String, TransportDialerFn>> {
    TRANSPORT_DIALER_REGISTRY.get_or_init(|| RwLock::new(HashMap::new()))
}

pub fn register_transport_dialer(
    protocol: impl Into<String>,
    dialer: TransportDialerFn,
) -> Result<()> {
    let registry = get_transport_dialer_registry();
    let mut map = registry
        .write()
        .map_err(|_| Error::Other("lock error".into()))?;
    let key = protocol.into();
    if map.contains_key(&key) {
        return Err(Error::Config(format!(
            "transport dialer {} already registered",
            key
        )));
    }
    map.insert(key, dialer);
    Ok(())
}

pub async fn dial_transport(
    dest: &Destination,
    stream_settings: Option<&MemoryStreamConfig>,
) -> Result<BoxStream> {
    let proto = match stream_settings {
        Some(s) if !s.protocol_name.is_empty() => s.protocol_name.as_str(),
        _ => match dest.network {
            Network::Tcp => "tcp",
            Network::Udp => "udp",
        },
    };

    let dialer_fn = {
        let registry = get_transport_dialer_registry();
        let map = registry
            .read()
            .map_err(|_| Error::Other("lock error".into()))?;
        map.get(proto).cloned()
    };

    if let Some(df) = dialer_fn {
        return df(dest, stream_settings).await;
    }

    // Default system fallback
    let sockopt = stream_settings.and_then(|s| s.socket_settings.as_ref());
    dial_system(dest, sockopt).await
}

pub async fn dial_system(dest: &Destination, sockopt: Option<&SocketOptions>) -> Result<BoxStream> {
    SystemDialer::dial(None, dest, sockopt).await
}

pub struct DefaultDialer;

#[async_trait]
impl Dialer for DefaultDialer {
    async fn dial(&self, dest: &Destination) -> Result<BoxStream> {
        dial_system(dest, None).await
    }
}
