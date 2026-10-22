// Module: transport\internet\tagged\taggedimpl\impl.rs
// 1:1 Rust implementation corresponding to Go transport\internet\tagged\taggedimpl\impl.go

use crate::common::errors::{Error, Result};
use crate::common::net::{BoxStream, Destination};
use crate::common::protocol::SessionContext;
use crate::features::routing::Dispatcher;
use crate::transport::internet::tagged::{DialFunc, set_dialer};
use std::sync::Arc;

pub async fn dial_tagged_outbound(
    dispatcher: Option<Arc<dyn Dispatcher>>,
    dest: Destination,
    tag: String,
) -> Result<BoxStream> {
    let d = dispatcher
        .ok_or_else(|| Error::Other("dispatcher required for tagged outbound dial".into()))?;
    let (client_stream, server_stream) = tokio::io::duplex(65536);
    let session = SessionContext::new(tag, dest);
    let d_clone = d.clone();
    tokio::spawn(async move {
        let _ = d_clone.dispatch(session, Box::pin(server_stream)).await;
    });
    Ok(Box::pin(client_stream))
}

pub async fn register_tagged_dialer() {
    let dialer: DialFunc = Arc::new(|d, dest, tag| Box::pin(dial_tagged_outbound(d, dest, tag)));
    set_dialer(dialer).await;
}
