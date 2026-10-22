// Module: transport\internet\tagged\tagged.rs
// 1:1 Rust implementation corresponding to Go transport\internet\tagged\tagged.go

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::common::errors::{Error, Result};
use crate::common::net::{BoxStream, Destination};
use crate::features::outbound::OutboundManager;
use crate::features::routing::Dispatcher;

pub type DialFunc = Arc<
    dyn Fn(
            Option<Arc<dyn Dispatcher>>,
            Destination,
            String,
        ) -> Pin<Box<dyn Future<Output = Result<BoxStream>> + Send>>
        + Send
        + Sync,
>;

lazy_static::lazy_static! {
    pub static ref DIALER: RwLock<Option<DialFunc>> = RwLock::new(None);
}

pub async fn set_dialer(dialer: DialFunc) {
    let mut guard = DIALER.write().await;
    *guard = Some(dialer);
}

pub async fn dial(
    dispatcher: Option<Arc<dyn Dispatcher>>,
    dest: Destination,
    tag: String,
) -> Result<BoxStream> {
    let guard = DIALER.read().await;
    if let Some(dialer) = guard.as_ref() {
        dialer(dispatcher, dest, tag).await
    } else {
        Err(Error::Other("tagged dialer not registered".into()))
    }
}

pub struct TaggedDialer {
    outbound_manager: Arc<dyn OutboundManager>,
}

impl TaggedDialer {
    pub fn new(outbound_manager: Arc<dyn OutboundManager>) -> Self {
        Self { outbound_manager }
    }

    pub async fn dial(&self, tag: &str, dest: Destination) -> Result<BoxStream> {
        let handler = self
            .outbound_manager
            .get_handler(tag)
            .await
            .ok_or_else(|| {
                Error::NotFound(format!("Tagged outbound handler not found: {}", tag))
            })?;

        let session = crate::common::protocol::SessionContext::new(tag.to_string(), dest);
        handler.connect(&session).await
    }
}
