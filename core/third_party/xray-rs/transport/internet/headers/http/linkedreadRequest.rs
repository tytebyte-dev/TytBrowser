// Module: transport\internet\headers\http\linkedreadRequest.rs
// 1:1 Rust implementation corresponding to Go transport\internet\headers\http\linkedreadRequest.go

use crate::common::errors::{Error, Result};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct ParsedRequest {
    pub method: String,
    pub uri: String,
    pub version: String,
    pub headers: HashMap<String, Vec<String>>,
}

pub fn read_request(data: &[u8]) -> Result<ParsedRequest> {
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);
    match req.parse(data) {
        Ok(httparse::Status::Complete(_)) => {
            let method = req.method.unwrap_or("GET").to_string();
            let uri = req.path.unwrap_or("/").to_string();
            let version = req
                .version
                .map(|v| format!("1.{}", v))
                .unwrap_or_else(|| "1.1".into());
            let mut header_map = HashMap::new();
            for h in req.headers {
                let name = h.name.to_string();
                let val = String::from_utf8_lossy(h.value).to_string();
                header_map.entry(name).or_insert_with(Vec::new).push(val);
            }
            Ok(ParsedRequest {
                method,
                uri,
                version,
                headers: header_map,
            })
        }
        Ok(httparse::Status::Partial) => Err(Error::Eof),
        Err(e) => Err(Error::Protocol(format!("malformed HTTP request: {}", e))),
    }
}
