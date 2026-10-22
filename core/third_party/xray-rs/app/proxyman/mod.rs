pub mod command;
pub mod config;
pub mod inbound;
pub mod outbound;

use crate::app::dispatcher::DefaultDispatcher;
use crate::common::errors::Result;
use crate::features::inbound::InboundHandler;
use crate::transport::internet::TcpHub;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tracing::{info, warn};

pub use config::{InboundHandlerConfig, KnownProtocols, OutboundHandlerConfig, SniffingConfig};
pub use inbound::DefaultInboundManager;
pub use outbound::DefaultOutboundManager;

pub struct InboundManager {
    inbounds: Vec<(SocketAddr, Arc<dyn InboundHandler>)>,
}

impl InboundManager {
    pub fn new(inbounds: Vec<(SocketAddr, Arc<dyn InboundHandler>)>) -> Self {
        Self { inbounds }
    }

    pub async fn start(&self, dispatcher: Arc<DefaultDispatcher>) -> Result<Vec<JoinHandle<()>>> {
        let mut tasks = Vec::new();

        for (addr, handler) in &self.inbounds {
            let listener = match TcpHub::listen(*addr).await {
                Ok(l) => l,
                Err(e) => {
                    warn!(
                        "Inbound [{}] could not bind to {} ({}). Continuing other inbounds...",
                        handler.tag(),
                        addr,
                        e
                    );
                    continue;
                }
            };

            info!("Inbound [{}] listening on {}", handler.tag(), addr);

            let handler = handler.clone();
            let dispatcher = dispatcher.clone();

            let task = tokio::spawn(async move {
                loop {
                    match listener.accept().await {
                        Ok((stream, remote_addr)) => {
                            let handler = handler.clone();
                            let dispatcher = dispatcher.clone();

                            tokio::spawn(async move {
                                match handler.handle_connection(stream, remote_addr).await {
                                    Ok(res) => {
                                        if let Err(e) =
                                            dispatcher.dispatch(res.stream, res.session).await
                                        {
                                            warn!("Dispatch error: {}", e);
                                        }
                                    }
                                    Err(e) => {
                                        warn!(
                                            "[{}] Handshake failed from {}: {}",
                                            handler.tag(),
                                            remote_addr,
                                            e
                                        );
                                    }
                                }
                            });
                        }
                        Err(e) => {
                            warn!("Accept error on inbound [{}]: {}", handler.tag(), e);
                            break;
                        }
                    }
                }
            });

            tasks.push(task);
        }

        Ok(tasks)
    }
}
