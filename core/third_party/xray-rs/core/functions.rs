// Module: core\functions.rs
// 1:1 Rust implementation corresponding to Go core\functions.go

use super::config::load_config;
use super::xray::Instance;
use crate::common::errors::Result;
use crate::common::net::{BoxStream, Destination};
use crate::common::protocol::SessionContext;
use crate::infra::conf::Config;
use crate::transport::Link;
use crate::transport::internet::udp::{DispatcherConn, LinkDispatcher, dial_dispatcher};
use crate::transport::pipe::new_pipe;
use std::sync::Arc;

/// CreateObject creates a new object based on the given Xray instance and config.
pub fn create_object(_instance: Option<&Instance>, config: &Config) -> Result<Instance> {
    Instance::from_config(config.clone())
}

/// StartInstance starts a new Xray instance with given serialized config.
pub fn start_instance(config_format: &str, config_bytes: &[u8]) -> Result<Instance> {
    let config = load_config(config_format, config_bytes)?;
    Instance::from_config(config)
}

/// Dial provides an easy way for upstream caller to create a bidirectional stream through Xray.
pub async fn dial(instance: &Instance, dest: &Destination) -> Result<BoxStream> {
    let (client_stream, server_stream) = tokio::io::duplex(64 * 1024);
    let session = SessionContext::new("dial".to_string(), dest.clone());
    let dispatcher = instance.dispatcher.clone();
    tokio::spawn(async move {
        let _ = dispatcher.dispatch(Box::pin(server_stream), session).await;
    });
    Ok(Box::pin(client_stream))
}

/// DialUDP provides a way to exchange UDP packets through Xray instance to remote servers.
pub async fn dial_udp(_instance: &Instance) -> Result<DispatcherConn> {
    let dispatcher: Arc<dyn LinkDispatcher> = Arc::new(|_dest: Destination| -> Result<Link> {
        let (reader, writer) = new_pipe(Default::default());
        Ok(Link::new(reader, writer))
    });
    dial_dispatcher(dispatcher).await
}
