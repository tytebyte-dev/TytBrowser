// Module: common\protocol\address.rs
// 1:1 Rust implementation corresponding to Go common\protocol\address.go

use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::common::errors::{Error, Result};
use crate::common::net::Address;

pub const ADDR_TYPE_IPV4: u8 = 0x01;
pub const ADDR_TYPE_DOMAIN: u8 = 0x02;
pub const ADDR_TYPE_IPV6: u8 = 0x03;

pub struct AddressParser {
    pub port_first: bool,
}

impl AddressParser {
    pub fn new(port_first: bool) -> Self {
        Self { port_first }
    }

    pub async fn read_address_port<R: AsyncRead + Unpin>(
        &self,
        reader: &mut R,
    ) -> Result<(Address, u16)> {
        if self.port_first {
            let port = reader.read_u16().await.map_err(Error::Io)?;
            let addr = self.read_address(reader).await?;
            Ok((addr, port))
        } else {
            let addr = self.read_address(reader).await?;
            let port = reader.read_u16().await.map_err(Error::Io)?;
            Ok((addr, port))
        }
    }

    pub async fn write_address_port<W: AsyncWrite + Unpin>(
        &self,
        writer: &mut W,
        addr: &Address,
        port: u16,
    ) -> Result<()> {
        if self.port_first {
            writer.write_u16(port).await.map_err(Error::Io)?;
            self.write_address(writer, addr).await?;
        } else {
            self.write_address(writer, addr).await?;
            writer.write_u16(port).await.map_err(Error::Io)?;
        }
        Ok(())
    }

    pub async fn read_address<R: AsyncRead + Unpin>(&self, reader: &mut R) -> Result<Address> {
        let addr_type = reader.read_u8().await.map_err(Error::Io)?;
        match addr_type {
            ADDR_TYPE_IPV4 => {
                let mut ip_bytes = [0u8; 4];
                reader.read_exact(&mut ip_bytes).await.map_err(Error::Io)?;
                Ok(Address::Ipv4(Ipv4Addr::from(ip_bytes)))
            }
            ADDR_TYPE_IPV6 => {
                let mut ip_bytes = [0u8; 16];
                reader.read_exact(&mut ip_bytes).await.map_err(Error::Io)?;
                Ok(Address::Ipv6(Ipv6Addr::from(ip_bytes)))
            }
            ADDR_TYPE_DOMAIN => {
                let len = reader.read_u8().await.map_err(Error::Io)? as usize;
                let mut domain_bytes = vec![0u8; len];
                reader
                    .read_exact(&mut domain_bytes)
                    .await
                    .map_err(Error::Io)?;
                let s = String::from_utf8(domain_bytes)
                    .map_err(|_| Error::Protocol("invalid utf8 domain".into()))?;
                Address::from_str(&s)
            }
            other => Err(Error::Protocol(format!("unknown address type: {}", other))),
        }
    }

    pub async fn write_address<W: AsyncWrite + Unpin>(
        &self,
        writer: &mut W,
        addr: &Address,
    ) -> Result<()> {
        match addr {
            Address::Ipv4(ip) => {
                writer.write_u8(ADDR_TYPE_IPV4).await.map_err(Error::Io)?;
                writer.write_all(&ip.octets()).await.map_err(Error::Io)?;
            }
            Address::Ipv6(ip) => {
                writer.write_u8(ADDR_TYPE_IPV6).await.map_err(Error::Io)?;
                writer.write_all(&ip.octets()).await.map_err(Error::Io)?;
            }
            Address::Domain(domain) => {
                writer.write_u8(ADDR_TYPE_DOMAIN).await.map_err(Error::Io)?;
                writer
                    .write_u8(domain.len() as u8)
                    .await
                    .map_err(Error::Io)?;
                writer
                    .write_all(domain.as_bytes())
                    .await
                    .map_err(Error::Io)?;
            }
        }
        Ok(())
    }
}
