// Module: proxy\vmess\outbound\outbound.rs
// 1:1 Rust implementation corresponding to Go proxy\vmess\outbound\outbound.go

use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::common::errors::Result;
use crate::common::net::{BoxStream, Destination, Network};
use crate::common::protocol::{RequestCommand, SessionContext};
use crate::features::outbound::OutboundHandler;
use crate::proxy::vmess::encoding::RequestHeader;
use crate::transport::internet::{TcpDialer, TlsClient};

pub struct Client {
    tag: String,
    server_addr: Destination,
    user_id: Uuid,
    tls_client: Option<Arc<TlsClient>>,
    tls_sni: Option<String>,
}

impl Client {
    pub fn new(
        tag: impl Into<String>,
        server_addr: Destination,
        user_id: Uuid,
        tls_client: Option<TlsClient>,
        tls_sni: Option<String>,
    ) -> Self {
        Self {
            tag: tag.into(),
            server_addr,
            user_id,
            tls_client: tls_client.map(Arc::new),
            tls_sni,
        }
    }
}

#[async_trait]
impl OutboundHandler for Client {
    fn tag(&self) -> &str {
        &self.tag
    }

    async fn connect(&self, session: &SessionContext) -> Result<BoxStream> {
        let tcp_stream = TcpDialer::dial(&self.server_addr).await?;

        let mut stream = if let Some(tls) = &self.tls_client {
            let sni = self
                .tls_sni
                .as_deref()
                .unwrap_or_else(|| match &self.server_addr.address {
                    crate::common::net::Address::Domain(d) => d.as_str(),
                    _ => "localhost",
                });
            tls.connect(sni, tcp_stream).await?
        } else {
            tcp_stream
        };

        let command = match session.destination.network {
            Network::Tcp => RequestCommand::Tcp,
            Network::Udp => RequestCommand::Udp,
        };

        let req = RequestHeader::new(self.user_id, command, session.destination.clone());
        req.encode(&mut stream).await?;

        Ok(stream)
    }
}

pub use Client as VmessOutboundClient;
