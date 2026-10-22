// Module: common\protocol\http\sniff.rs
// 1:1 Rust implementation corresponding to Go common\protocol\http\sniff.go

use crate::common::errors::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpVersion {
    Http1,
    Http2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SniffHeader {
    pub version: HttpVersion,
    pub host: String,
    pub path: Option<String>,
    pub method: Option<String>,
}

impl SniffHeader {
    pub fn protocol(&self) -> &'static str {
        match self.version {
            HttpVersion::Http1 => "http1",
            HttpVersion::Http2 => "http2",
        }
    }

    pub fn domain(&self) -> &str {
        &self.host
    }
}

const METHODS: [&str; 7] = ["get", "post", "head", "put", "delete", "options", "connect"];

pub fn sniff_http(bytes: &[u8]) -> Result<SniffHeader> {
    let s =
        std::str::from_utf8(bytes).map_err(|_| Error::Protocol("invalid utf8 for http".into()))?;
    let mut lines = s.lines();

    let first_line = lines
        .next()
        .ok_or_else(|| Error::Protocol("empty http request".into()))?;
    let mut parts = first_line.split_whitespace();
    let method = parts
        .next()
        .ok_or_else(|| Error::Protocol("no method".into()))?;

    if !METHODS.iter().any(|m| m.eq_ignore_ascii_case(method)) {
        return Err(Error::Protocol("not http method".into()));
    }

    let path = parts.next().map(|s| s.to_string());
    let mut host = None;

    for line in lines {
        if line.is_empty() {
            break;
        }
        if let Some((k, v)) = line.split_once(':')
            && k.trim().eq_ignore_ascii_case("host")
        {
            let h = v.trim();
            let clean_host = if let Some((h_only, _port)) = h.rsplit_once(':') {
                h_only
            } else {
                h
            };
            host = Some(clean_host.to_string());
            break;
        }
    }

    if let Some(h) = host {
        Ok(SniffHeader {
            version: HttpVersion::Http1,
            host: h,
            path,
            method: Some(method.to_uppercase()),
        })
    } else {
        Err(Error::Protocol("host header not found".into()))
    }
}
