use crate::app::dispatcher::DefaultDispatcher;
use crate::common::errors::Result;
use crate::common::net::Destination;
use crate::common::protocol::SessionContext;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};

pub struct LoopbackOutbound {
    pub inbound_tag: String,
    dispatcher: Arc<DefaultDispatcher>,
}

impl LoopbackOutbound {
    pub fn new(inbound_tag: String, dispatcher: Arc<DefaultDispatcher>) -> Self {
        Self {
            inbound_tag,
            dispatcher,
        }
    }

    pub async fn process<S>(&self, stream: S, dest: Destination) -> Result<()>
    where
        S: AsyncRead + AsyncWrite + Send + Sync + Unpin + 'static,
    {
        let session = SessionContext::new(self.inbound_tag.clone(), dest);
        self.dispatcher.dispatch(Box::pin(stream), session).await
    }
}

#[async_trait::async_trait]
impl crate::features::outbound::OutboundHandler for LoopbackOutbound {
    fn tag(&self) -> &str {
        &self.inbound_tag
    }

    async fn connect(&self, session: &SessionContext) -> Result<crate::common::net::BoxStream> {
        let (client, server) = tokio::io::duplex(64 * 1024);
        let mut loop_session =
            SessionContext::new(self.inbound_tag.clone(), session.destination.clone());
        loop_session.source = session.source;
        let dispatcher = self.dispatcher.clone();
        tokio::spawn(async move {
            let _ = dispatcher.dispatch(Box::pin(server), loop_session).await;
        });
        Ok(Box::pin(client))
    }
}
