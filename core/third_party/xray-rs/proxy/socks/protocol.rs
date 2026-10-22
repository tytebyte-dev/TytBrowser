use crate::common::errors::{Error, Result};
use crate::common::net::{Address, Destination};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub const SOCKS5_VERSION: u8 = 0x05;
pub const SOCKS4_VERSION: u8 = 0x04;

pub const CMD_TCP_CONNECT: u8 = 0x01;
pub const CMD_TCP_BIND: u8 = 0x02;
pub const CMD_UDP_ASSOCIATE: u8 = 0x03;

pub const AUTH_NO_AUTH: u8 = 0x00;
pub const AUTH_PASSWORD: u8 = 0x02;
pub const AUTH_NO_MATCH: u8 = 0xFF;

pub const STATUS_SUCCESS: u8 = 0x00;
pub const STATUS_COMMAND_NOT_SUPPORTED: u8 = 0x07;

pub struct SocksProtocol;

impl SocksProtocol {
    pub async fn client_handshake<S: AsyncRead + AsyncWrite + Unpin>(
        stream: &mut S,
        target: &Destination,
    ) -> Result<()> {
        // 1. Client greeting: version 5, 1 auth method (No Auth = 0x00)
        stream
            .write_all(&[SOCKS5_VERSION, 0x01, AUTH_NO_AUTH])
            .await
            .map_err(Error::Io)?;
        stream.flush().await.map_err(Error::Io)?;

        // 2. Server choice
        let mut resp = [0u8; 2];
        stream.read_exact(&mut resp).await.map_err(Error::Io)?;
        if resp[0] != SOCKS5_VERSION || resp[1] != AUTH_NO_AUTH {
            return Err(Error::Protocol(format!(
                "SOCKS5 auth negotiation failed: {:?}",
                resp
            )));
        }

        // 3. Connect request: [0x05, 0x01, 0x00, ATYP, ADDR, PORT]
        let mut req = vec![SOCKS5_VERSION, CMD_TCP_CONNECT, 0x00];
        match &target.address {
            Address::Ipv4(v4) => {
                req.push(0x01);
                req.extend_from_slice(&v4.octets());
            }
            Address::Ipv6(v6) => {
                req.push(0x04);
                req.extend_from_slice(&v6.octets());
            }
            Address::Domain(domain) => {
                req.push(0x03);
                req.push(domain.len() as u8);
                req.extend_from_slice(domain.as_bytes());
            }
        }
        req.extend_from_slice(&target.port.to_be_bytes());

        stream.write_all(&req).await.map_err(Error::Io)?;
        stream.flush().await.map_err(Error::Io)?;

        // 4. Server reply: [0x05, REP, 0x00, ATYP, ADDR, PORT]
        let mut reply_hdr = [0u8; 4];
        stream.read_exact(&mut reply_hdr).await.map_err(Error::Io)?;
        if reply_hdr[0] != SOCKS5_VERSION {
            return Err(Error::Protocol(format!(
                "Invalid SOCKS5 reply version: {}",
                reply_hdr[0]
            )));
        }
        if reply_hdr[1] != STATUS_SUCCESS {
            return Err(Error::Protocol(format!(
                "SOCKS5 server replied error status: 0x{:02X}",
                reply_hdr[1]
            )));
        }

        // Skip bound addr & port
        match reply_hdr[3] {
            0x01 => {
                let mut b = [0u8; 4 + 2];
                stream.read_exact(&mut b).await.map_err(Error::Io)?;
            }
            0x04 => {
                let mut b = [0u8; 16 + 2];
                stream.read_exact(&mut b).await.map_err(Error::Io)?;
            }
            0x03 => {
                let len = stream.read_u8().await.map_err(Error::Io)? as usize;
                let mut b = vec![0u8; len + 2];
                stream.read_exact(&mut b).await.map_err(Error::Io)?;
            }
            other => {
                return Err(Error::Protocol(format!(
                    "Unknown ATYP in reply: 0x{:02X}",
                    other
                )));
            }
        }

        Ok(())
    }
}
