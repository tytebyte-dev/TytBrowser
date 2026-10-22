use crate::common::errors::{Error, Result};
use tokio::io::{AsyncRead, AsyncReadExt};

pub const MAX_ADDRESS_LENGTH: usize = 2048;
pub const MAX_PADDING_LENGTH: usize = 4096;

pub struct QuicVarint;

impl QuicVarint {
    pub async fn read<R: AsyncRead + Unpin>(reader: &mut R) -> Result<u64> {
        let first = reader.read_u8().await?;
        let len = 1 << (first >> 6);
        let mut val = (first & 0x3f) as u64;
        for _ in 1..len {
            let next = reader.read_u8().await?;
            val = (val << 8) | (next as u64);
        }
        Ok(val)
    }

    pub fn write(val: u64, buf: &mut Vec<u8>) {
        if val <= 63 {
            buf.push(val as u8);
        } else if val <= 16383 {
            buf.push(0x40 | ((val >> 8) as u8));
            buf.push(val as u8);
        } else if val <= 1073741823 {
            buf.push(0x80 | ((val >> 24) as u8));
            buf.push((val >> 16) as u8);
            buf.push((val >> 8) as u8);
            buf.push(val as u8);
        } else {
            buf.push(0xc0 | ((val >> 56) as u8));
            buf.push((val >> 48) as u8);
            buf.push((val >> 40) as u8);
            buf.push((val >> 32) as u8);
            buf.push((val >> 24) as u8);
            buf.push((val >> 16) as u8);
            buf.push((val >> 8) as u8);
            buf.push(val as u8);
        }
    }
}

#[derive(Debug, Clone)]
pub struct TcpRequest {
    pub address: String,
    pub padding_len: usize,
}

impl TcpRequest {
    pub fn new(address: impl Into<String>, padding_len: usize) -> Self {
        Self {
            address: address.into(),
            padding_len,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        let addr_bytes = self.address.as_bytes();
        QuicVarint::write(addr_bytes.len() as u64, &mut buf);
        buf.extend_from_slice(addr_bytes);
        QuicVarint::write(self.padding_len as u64, &mut buf);
        if self.padding_len > 0 {
            buf.extend(std::iter::repeat_n(0u8, self.padding_len));
        }
        buf
    }

    pub async fn decode<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Self> {
        let addr_len = QuicVarint::read(reader).await? as usize;
        if addr_len == 0 || addr_len > MAX_ADDRESS_LENGTH {
            return Err(Error::Protocol("Invalid Hysteria address length".into()));
        }
        let mut addr_buf = vec![0u8; addr_len];
        reader.read_exact(&mut addr_buf).await?;
        let address = String::from_utf8(addr_buf)
            .map_err(|e| Error::Protocol(format!("Invalid UTF-8 in address: {}", e)))?;

        let padding_len = QuicVarint::read(reader).await? as usize;
        if padding_len > MAX_PADDING_LENGTH {
            return Err(Error::Protocol("Invalid Hysteria padding length".into()));
        }
        if padding_len > 0 {
            let mut pad_buf = vec![0u8; padding_len];
            reader.read_exact(&mut pad_buf).await?;
        }

        Ok(Self {
            address,
            padding_len,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hysteria_tcp_request_roundtrip() {
        let req = TcpRequest::new("example.com:443", 32);
        let encoded = req.encode();

        let mut cursor = std::io::Cursor::new(encoded);
        let decoded = TcpRequest::decode(&mut cursor)
            .await
            .expect("decode success");

        assert_eq!(decoded.address, "example.com:443");
        assert_eq!(decoded.padding_len, 32);
    }
}
