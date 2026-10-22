// Module: common\protocol\tls\sniff.rs
// 1:1 Rust implementation corresponding to Go common\protocol\tls\sniff.go

use crate::common::errors::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlsSniffHeader {
    pub domain: String,
}

impl TlsSniffHeader {
    pub fn protocol(&self) -> &'static str {
        "tls"
    }

    pub fn domain(&self) -> &str {
        &self.domain
    }
}

pub fn sniff_tls(b: &[u8]) -> Result<TlsSniffHeader> {
    if b.len() < 5 {
        return Err(Error::Protocol("TLS payload too short".into()));
    }

    if b[0] != 0x16
    /* TLS Handshake */
    {
        return Err(Error::Protocol("not TLS handshake".into()));
    }

    let major = b[1];
    if major != 3 {
        return Err(Error::Protocol("not valid TLS version".into()));
    }

    let header_len = u16::from_be_bytes([b[3], b[4]]) as usize;
    if 5 + header_len > b.len() {
        return Err(Error::Protocol("incomplete TLS record".into()));
    }

    let data = &b[5..5 + header_len];
    let domain = read_client_hello(data)?;
    Ok(TlsSniffHeader { domain })
}

pub fn read_client_hello(mut data: &[u8]) -> Result<String> {
    if data.len() < 42 {
        return Err(Error::Protocol("client hello too short".into()));
    }
    let session_id_len = data[38] as usize;
    if session_id_len > 32 || data.len() < 39 + session_id_len {
        return Err(Error::Protocol("invalid session id len".into()));
    }
    data = &data[39 + session_id_len..];

    if data.len() < 2 {
        return Err(Error::Protocol("no cipher suite len".into()));
    }
    let cipher_suite_len = u16::from_be_bytes([data[0], data[1]]) as usize;
    if data.len() < 2 + cipher_suite_len {
        return Err(Error::Protocol("invalid cipher suite len".into()));
    }
    data = &data[2 + cipher_suite_len..];

    if data.is_empty() {
        return Err(Error::Protocol("no compression method len".into()));
    }
    let comp_len = data[0] as usize;
    if data.len() < 1 + comp_len {
        return Err(Error::Protocol("invalid compression method len".into()));
    }
    data = &data[1 + comp_len..];

    if data.len() < 2 {
        return Err(Error::Protocol("no extensions len".into()));
    }
    let ext_len = u16::from_be_bytes([data[0], data[1]]) as usize;
    data = &data[2..];
    if ext_len != data.len() {
        return Err(Error::Protocol("extensions length mismatch".into()));
    }

    while data.len() >= 4 {
        let ext_type = u16::from_be_bytes([data[0], data[1]]);
        let length = u16::from_be_bytes([data[2], data[3]]) as usize;
        data = &data[4..];
        if data.len() < length {
            return Err(Error::Protocol("extension truncated".into()));
        }

        if ext_type == 0x00 {
            // Server Name Indication
            let mut sni_data = &data[..length];
            if sni_data.len() < 2 {
                return Err(Error::Protocol("sni data too short".into()));
            }
            let names_len = u16::from_be_bytes([sni_data[0], sni_data[1]]) as usize;
            sni_data = &sni_data[2..];
            if sni_data.len() < names_len {
                return Err(Error::Protocol("sni names len mismatch".into()));
            }

            while sni_data.len() >= 3 {
                let name_type = sni_data[0];
                let name_len = u16::from_be_bytes([sni_data[1], sni_data[2]]) as usize;
                sni_data = &sni_data[3..];
                if sni_data.len() < name_len {
                    return Err(Error::Protocol("sni name truncated".into()));
                }

                if name_type == 0 {
                    let domain_bytes = &sni_data[..name_len];
                    let domain = std::str::from_utf8(domain_bytes)
                        .map_err(|_| Error::Protocol("invalid utf8 sni".into()))?;
                    return Ok(domain.to_string());
                }
                sni_data = &sni_data[name_len..];
            }
        }
        data = &data[length..];
    }

    Err(Error::Protocol("SNI extension not found".into()))
}
