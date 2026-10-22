use crate::common::errors::{Error, Result};
use crate::common::net::{Address, Destination};
use hkdf::Hkdf;
use sha1::Sha1;
use std::net::{Ipv4Addr, Ipv6Addr};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum CipherType {
    None,
    #[default]
    Aes128Gcm,
    Aes256Gcm,
    ChaCha20Poly1305,
}

pub fn derive_subkey(key: &[u8], salt: &[u8], subkey: &mut [u8]) -> Result<()> {
    let hk = Hkdf::<Sha1>::new(Some(salt), key);
    hk.expand(b"ss-subkey", subkey)
        .map_err(|e| Error::Crypto(format!("HKDF subkey derivation failed: {:?}", e)))
}

pub async fn read_target_address<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Destination> {
    let atyp = reader.read_u8().await?;
    let address = match atyp {
        0x01 => {
            let mut ip_bytes = [0u8; 4];
            reader.read_exact(&mut ip_bytes).await?;
            Address::Ipv4(Ipv4Addr::from(ip_bytes))
        }
        0x03 => {
            let len = reader.read_u8().await? as usize;
            let mut domain_bytes = vec![0u8; len];
            reader.read_exact(&mut domain_bytes).await?;
            let domain = String::from_utf8(domain_bytes).map_err(|e| {
                Error::Protocol(format!("Invalid domain string in Shadowsocks: {}", e))
            })?;
            Address::Domain(domain)
        }
        0x04 => {
            let mut ip_bytes = [0u8; 16];
            reader.read_exact(&mut ip_bytes).await?;
            Address::Ipv6(Ipv6Addr::from(ip_bytes))
        }
        _ => {
            return Err(Error::Protocol(format!(
                "Unsupported Shadowsocks ATYP: {}",
                atyp
            )));
        }
    };

    let port = reader.read_u16().await?;
    Ok(Destination::new(address, port))
}

pub async fn write_target_address<W: AsyncWrite + Unpin>(
    writer: &mut W,
    dest: &Destination,
) -> Result<()> {
    match &dest.address {
        Address::Ipv4(ip) => {
            writer.write_u8(0x01).await?;
            writer.write_all(&ip.octets()).await?;
        }
        Address::Domain(domain) => {
            writer.write_u8(0x03).await?;
            let bytes = domain.as_bytes();
            if bytes.len() > 255 {
                return Err(Error::Protocol("Domain too long".into()));
            }
            writer.write_u8(bytes.len() as u8).await?;
            writer.write_all(bytes).await?;
        }
        Address::Ipv6(ip) => {
            writer.write_u8(0x04).await?;
            writer.write_all(&ip.octets()).await?;
        }
    }
    writer.write_u16(dest.port).await?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShadowsocksUdpPacket {
    pub destination: Destination,
    pub payload: Vec<u8>,
}

impl ShadowsocksUdpPacket {
    pub fn new(destination: Destination, payload: Vec<u8>) -> Self {
        Self {
            destination,
            payload,
        }
    }

    pub async fn encode<W: AsyncWrite + Unpin>(&self, writer: &mut W) -> Result<()> {
        write_target_address(writer, &self.destination).await?;
        writer.write_all(&self.payload).await?;
        writer.flush().await?;
        Ok(())
    }

    pub async fn decode<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Self> {
        let destination = read_target_address(reader).await?;
        let mut payload = Vec::new();
        reader.read_to_end(&mut payload).await?;
        Ok(Self {
            destination,
            payload,
        })
    }
}
