use crate::app::commander::Commander;
use crate::common::errors::Result;
use crate::common::net::BoxStream;
use crate::common::protocol::SessionContext;
use crate::features::outbound::OutboundHandler;
use async_trait::async_trait;
use std::sync::Arc;

pub struct CommanderOutbound {
    tag: String,
    commander: Arc<Commander>,
}

impl CommanderOutbound {
    pub fn new(tag: impl Into<String>, commander: Arc<Commander>) -> Self {
        Self {
            tag: tag.into(),
            commander,
        }
    }

    pub fn commander(&self) -> &Arc<Commander> {
        &self.commander
    }
}

#[async_trait]
impl OutboundHandler for CommanderOutbound {
    fn tag(&self) -> &str {
        &self.tag
    }

    async fn connect(&self, _session: &SessionContext) -> Result<BoxStream> {
        let (client, _) = tokio::io::duplex(1024);
        Ok(Box::pin(client))
    }

    async fn start(&self) -> Result<()> {
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        Ok(())
    }
}
