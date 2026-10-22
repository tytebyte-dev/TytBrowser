#[path = "config.pb.rs"]
pub mod config_pb;
pub mod dns;
#[cfg(test)]
pub mod dns_test;

use crate::app::dns::DnsClient;
use crate::common::errors::Result;
use crate::common::net::Destination;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub struct DefaultDnsHandler {
    pub tag: String,
}

impl DefaultDnsHandler {
    pub fn new(tag: impl Into<String>) -> Self {
        Self { tag: tag.into() }
    }
}

use crate::common::net::BoxStream;
use crate::common::protocol::SessionContext;
use crate::features::outbound::OutboundHandler;
use async_trait::async_trait;

#[derive(Clone)]
pub struct DnsOutbound {
    pub tag: String,
    dns_client: Arc<DnsClient>,
}

impl DnsOutbound {
    pub fn new(tag: String, dns_client: Arc<DnsClient>) -> Self {
        Self { tag, dns_client }
    }

    pub async fn process<S>(&self, mut stream: S, _dest: Destination) -> Result<()>
    where
        S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        let mut query = vec![0u8; 512];
        let n = stream.read(&mut query).await?;
        if n == 0 {
            return Ok(());
        }

        // Basic DNS query parsing: skip header (12 bytes) and extract query domain
        if n > 12 {
            let mut pos = 12;
            let mut domain_parts = Vec::new();
            while pos < n && query[pos] != 0 {
                let len = query[pos] as usize;
                pos += 1;
                if pos + len <= n {
                    if let Ok(part) = std::str::from_utf8(&query[pos..pos + len]) {
                        domain_parts.push(part);
                    }
                    pos += len;
                } else {
                    break;
                }
            }

            let domain = domain_parts.join(".");
            if !domain.is_empty()
                && let Ok(ips) = self.dns_client.lookup_ip(&domain).await
                && let Some(ip) = ips.first()
            {
                let mut response = query[0..(pos + 5).min(n)].to_vec();
                // Set QR flag to 1 (response)
                if response.len() >= 4 {
                    response[2] |= 0x80;
                    response[3] |= 0x80;
                }
                if response.len() >= 8 {
                    // Set Answer count = 1
                    response[6] = 0;
                    response[7] = 1;
                }

                // Answer RR: pointer to name (0xc00c)
                response.extend_from_slice(&[0xc0, 0x0c]);
                match ip {
                    IpAddr::V4(v4) => {
                        response.extend_from_slice(&[0x00, 0x01]); // Type A
                        response.extend_from_slice(&[0x00, 0x01]); // Class IN
                        response.extend_from_slice(&[0x00, 0x00, 0x00, 0x3c]); // TTL 60s
                        response.extend_from_slice(&[0x00, 0x04]); // Data len 4
                        response.extend_from_slice(&v4.octets());
                    }
                    IpAddr::V6(v6) => {
                        response.extend_from_slice(&[0x00, 0x1c]); // Type AAAA
                        response.extend_from_slice(&[0x00, 0x01]); // Class IN
                        response.extend_from_slice(&[0x00, 0x00, 0x00, 0x3c]); // TTL 60s
                        response.extend_from_slice(&[0x00, 0x10]); // Data len 16
                        response.extend_from_slice(&v6.octets());
                    }
                }

                stream.write_all(&response).await?;
                stream.flush().await?;
                return Ok(());
            }
        }

        Ok(())
    }
}

#[async_trait]
impl OutboundHandler for DnsOutbound {
    fn tag(&self) -> &str {
        &self.tag
    }

    async fn connect(&self, session: &SessionContext) -> Result<BoxStream> {
        let (client_io, server_io) = tokio::io::duplex(4096);
        let handler = self.clone();
        let dest = session.destination.clone();
        tokio::spawn(async move {
            if let Err(e) = handler.process(server_io, dest).await {
                tracing::debug!("DNS outbound process ended: {}", e);
            }
        });
        Ok(Box::pin(client_io))
    }
}
