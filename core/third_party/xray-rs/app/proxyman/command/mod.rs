pub mod command;
pub mod doc;
pub mod handler;

#[cfg(test)]
pub mod command_test;

use crate::app::commander::Service;
use crate::app::proxyman::inbound::DefaultInboundManager;
use crate::app::proxyman::outbound::DefaultOutboundManager;
use crate::common::errors::Result;
use crate::features::inbound::InboundManager;
use crate::features::outbound::OutboundManager;
use std::sync::Arc;

pub use command::ProxymanCommandService;

pub struct HandlerService {
    inbound_manager: Option<Arc<DefaultInboundManager>>,
    outbound_manager: Option<Arc<DefaultOutboundManager>>,
}

impl HandlerService {
    pub fn new(
        inbound_manager: Option<Arc<DefaultInboundManager>>,
        outbound_manager: Option<Arc<DefaultOutboundManager>>,
    ) -> Self {
        Self {
            inbound_manager,
            outbound_manager,
        }
    }

    pub async fn remove_inbound(&self, tag: &str) -> Result<bool> {
        if let Some(im) = &self.inbound_manager {
            match im.remove_handler(tag).await {
                Ok(_) => Ok(true),
                Err(_) => Ok(false),
            }
        } else {
            Ok(false)
        }
    }

    pub async fn remove_outbound(&self, tag: &str) -> Result<bool> {
        if let Some(om) = &self.outbound_manager {
            match om.remove_handler(tag).await {
                Ok(_) => Ok(true),
                Err(_) => Ok(false),
            }
        } else {
            Ok(false)
        }
    }
}

impl Service for HandlerService {
    fn service_name(&self) -> &str {
        "xray.core.app.proxyman.command.HandlerService"
    }
}
