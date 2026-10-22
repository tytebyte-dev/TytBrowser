use crate::common::errors::Result;
use crate::common::net::{BoxStream, Destination};
use crate::common::protocol::SessionContext;
use crate::features::outbound::OutboundHandler;
use crate::proxy::socks::protocol::SocksProtocol;
use async_trait::async_trait;

pub struct Client {
    tag: String,
    server: Destination,
}

impl Client {
    pub fn new(tag: impl Into<String>, server: Destination) -> Self {
        Self {
            tag: tag.into(),
            server,
        }
    }
}

#[async_trait]
impl OutboundHandler for Client {
    fn tag(&self) -> &str {
        &self.tag
    }

    async fn connect(&self, session: &SessionContext) -> Result<BoxStream> {
        let mut stream = crate::transport::internet::TcpDialer::dial(&self.server).await?;
        SocksProtocol::client_handshake(&mut stream, &session.destination).await?;
        Ok(stream)
    }
}
