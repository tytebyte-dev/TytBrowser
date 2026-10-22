// Module: proxy\trojan\client.rs
// 1:1 Rust implementation corresponding to Go proxy\trojan\client.go

use async_trait::async_trait;
use std::sync::Arc;

use crate::common::errors::Result;
use crate::common::net::{BoxStream, Destination, Network};
use crate::common::protocol::{RequestCommand, SessionContext};
use crate::features::outbound::OutboundHandler;
use crate::proxy::trojan::protocol::RequestHeader;
use crate::transport::internet::{TcpDialer, TlsClient, WebSocketStream};

pub struct Client {
    tag: String,
    server_addr: Destination,
    password: String,
    tls_client: Option<Arc<TlsClient>>,
    tls_sni: Option<String>,
    ws_path: Option<String>,
    ws_host: Option<String>,
}

impl Client {
    pub fn new(
        tag: impl Into<String>,
        server_addr: Destination,
        password: impl Into<String>,
        tls_client: Option<TlsClient>,
        tls_sni: Option<String>,
    ) -> Self {
        Self {
            tag: tag.into(),
            server_addr,
            password: password.into(),
            tls_client: tls_client.map(Arc::new),
            tls_sni,
            ws_path: None,
            ws_host: None,
        }
    }

    pub fn with_websocket(mut self, path: impl Into<String>, host: Option<String>) -> Self {
        self.ws_path = Some(path.into());
        self.ws_host = host;
        self
    }
}

#[async_trait]
impl OutboundHandler for Client {
    fn tag(&self) -> &str {
        &self.tag
    }

    async fn connect(&self, session: &SessionContext) -> Result<BoxStream> {
        let tcp_stream = TcpDialer::dial(&self.server_addr).await?;

        let sni = self
            .tls_sni
            .as_deref()
            .unwrap_or_else(|| match &self.server_addr.address {
                crate::common::net::Address::Domain(d) => d.as_str(),
                _ => "localhost",
            });

        // 1. Optional TLS Layer
        let stream = if let Some(tls) = &self.tls_client {
            tls.connect(sni, tcp_stream).await?
        } else {
            tcp_stream
        };

        // 2. Optional WebSocket Layer
        let mut stream = if let Some(path) = &self.ws_path {
            let dest_host = match &self.server_addr.address {
                crate::common::net::Address::Domain(d) => d.clone(),
                crate::common::net::Address::Ipv4(ip) => ip.to_string(),
                crate::common::net::Address::Ipv6(ip) => format!("[{}]", ip),
            };
            let host_hdr = self.ws_host.as_deref().unwrap_or(sni);
            let scheme = if self.tls_client.is_some() {
                "wss"
            } else {
                "ws"
            };
            let ws_url = format!("{}://{}{}", scheme, dest_host, path);
            WebSocketStream::client_handshake(&ws_url, Some(host_hdr), stream).await?
        } else {
            stream
        };

        // 3. Trojan Protocol Header
        let command = match session.destination.network {
            Network::Tcp => RequestCommand::Tcp,
            Network::Udp => RequestCommand::Udp,
        };

        let req = RequestHeader::new(&self.password, command, session.destination.clone());
        req.encode(&mut stream).await?;

        Ok(stream)
    }
}

pub use Client as TrojanClient;
