// Module: proxy\vless\outbound\outbound.rs
// 1:1 Rust implementation corresponding to Go proxy\vless\outbound\outbound.go

use async_trait::async_trait;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use uuid::Uuid;

use crate::common::errors::Result;
use crate::common::net::{BoxStream, Destination, Network};
use crate::common::protocol::{RequestCommand, SessionContext};
use crate::features::outbound::OutboundHandler;
use crate::proxy::vless::encoding::{Addons, RequestHeader};
use crate::proxy::vless::flow::{FLOW_VISION, VisionStream};
use crate::transport::internet::reality::RealityClient;
use crate::transport::internet::{TcpDialer, TlsClient, WebSocketStream};

pub struct Client {
    tag: String,
    server_addr: Destination,
    user_id: Uuid,
    flow: Option<String>,
    encryption: Option<String>,
    tls_client: Option<Arc<TlsClient>>,
    tls_sni: Option<String>,
    reality_client: Option<Arc<RealityClient>>,
    ws_path: Option<String>,
    ws_host: Option<String>,
    fragment_cfg: Option<crate::transport::internet::finalmask::fragment::Config>,
}

impl Client {
    pub fn new(
        tag: impl Into<String>,
        server_addr: Destination,
        user_id: Uuid,
        tls_client: Option<TlsClient>,
        tls_sni: Option<String>,
        ws_path: Option<String>,
        ws_host: Option<String>,
    ) -> Self {
        Self {
            tag: tag.into(),
            server_addr,
            user_id,
            flow: None,
            encryption: None,
            tls_client: tls_client.map(Arc::new),
            tls_sni,
            reality_client: None,
            ws_path,
            ws_host,
            fragment_cfg: None,
        }
    }

    pub fn with_fragment(
        mut self,
        cfg: crate::transport::internet::finalmask::fragment::Config,
    ) -> Self {
        self.fragment_cfg = Some(cfg);
        self
    }

    pub fn with_flow(mut self, flow: impl Into<String>) -> Self {
        self.flow = Some(flow.into());
        self
    }

    pub fn with_encryption(mut self, enc: impl Into<String>) -> Self {
        self.encryption = Some(enc.into());
        self
    }

    pub fn with_reality(mut self, client: RealityClient) -> Self {
        self.reality_client = Some(Arc::new(client));
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

        let tcp_stream: BoxStream = if let Some(ref f_cfg) = self.fragment_cfg {
            Box::pin(
                crate::transport::internet::finalmask::fragment::FragmentConn::new_client(
                    f_cfg.clone(),
                    tcp_stream,
                ),
            )
        } else {
            tcp_stream
        };

        let sni = self
            .tls_sni
            .as_deref()
            .unwrap_or_else(|| match &self.server_addr.address {
                crate::common::net::Address::Domain(d) => d.as_str(),
                _ => "localhost",
            });

        // 1. Optional TLS or Reality Layer
        let stream = if let Some(reality) = &self.reality_client {
            if let Some(tls) = &self.tls_client {
                tls.connect(&reality.server_name, tcp_stream).await?
            } else {
                tcp_stream
            }
        } else if let Some(tls) = &self.tls_client {
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
            let scheme = if self.tls_client.is_some() || self.reality_client.is_some() {
                "wss"
            } else {
                "ws"
            };
            let ws_url = format!("{}://{}{}", scheme, dest_host, path);
            WebSocketStream::client_handshake(&ws_url, Some(host_hdr), stream).await?
        } else {
            stream
        };

        // 3. VLESS Protocol Layer: Send Request Header with Addons
        let command = match session.destination.network {
            Network::Tcp => RequestCommand::Tcp,
            Network::Udp => RequestCommand::Udp,
        };

        let mut req = RequestHeader::new(self.user_id, command, session.destination.clone());
        if let Some(ref flow) = self.flow {
            req.addons = Addons::new(flow.clone());
        }
        req.encode(&mut stream).await?;

        // 4. Wrap with non-blocking VlessStream to strip response header
        let vless_stream: BoxStream = Box::pin(VlessStream::new(stream));

        // 5. Optional XTLS-Vision flow wrapping
        let final_stream: BoxStream = if self.flow.as_deref() == Some(FLOW_VISION) {
            Box::pin(VisionStream::new(
                vless_stream,
                crate::proxy::vless::flow::VisionContext::new(
                    self.user_id.into_bytes().to_vec(),
                    true,
                ),
                false,
            ))
        } else {
            vless_stream
        };

        Ok(final_stream)
    }
}

pub struct VlessStream {
    inner: BoxStream,
    header_parsed: bool,
    pending_buf: Vec<u8>,
}

impl VlessStream {
    pub fn new(inner: BoxStream) -> Self {
        Self {
            inner,
            header_parsed: false,
            pending_buf: Vec::with_capacity(512),
        }
    }
}

impl AsyncRead for VlessStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        loop {
            if self.header_parsed {
                if !self.pending_buf.is_empty() {
                    let to_write = std::cmp::min(buf.remaining(), self.pending_buf.len());
                    buf.put_slice(&self.pending_buf[..to_write]);
                    self.pending_buf.drain(..to_write);
                    return Poll::Ready(Ok(()));
                }
                return Pin::new(&mut self.inner).poll_read(cx, buf);
            }

            let mut temp_buf = [0u8; 1024];
            let mut temp_read_buf = ReadBuf::new(&mut temp_buf);

            match Pin::new(&mut self.inner).poll_read(cx, &mut temp_read_buf) {
                Poll::Ready(Ok(())) => {
                    let filled = temp_read_buf.filled();
                    if filled.is_empty() {
                        return Poll::Ready(Ok(()));
                    }
                    self.pending_buf.extend_from_slice(filled);

                    if self.pending_buf.len() >= 2 {
                        let addons_len = self.pending_buf[1] as usize;
                        let header_len = 2 + addons_len;

                        if self.pending_buf.len() >= header_len {
                            self.header_parsed = true;
                            self.pending_buf.drain(..header_len);

                            if !self.pending_buf.is_empty() {
                                let to_write =
                                    std::cmp::min(buf.remaining(), self.pending_buf.len());
                                buf.put_slice(&self.pending_buf[..to_write]);
                                self.pending_buf.drain(..to_write);
                                return Poll::Ready(Ok(()));
                            }
                            // Loop around to read payload from inner
                            continue;
                        }
                    }
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

impl AsyncWrite for VlessStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

pub use Client as VlessOutboundClient;
pub use Client as Handler;
