// Module: common\protocol\http\headers.rs
// 1:1 Rust implementation corresponding to Go common\protocol\http\headers.go

use std::collections::HashMap;
use std::str::FromStr;

use crate::common::errors::Result;
use crate::common::net::{Address, Destination};

pub fn parse_x_forwarded_for(headers: &HashMap<String, String>) -> Vec<Address> {
    for (k, v) in headers {
        if k.eq_ignore_ascii_case("x-forwarded-for") {
            return v
                .split(',')
                .filter_map(|s| Address::from_str(s.trim()).ok())
                .collect();
        }
    }
    Vec::new()
}

pub fn remove_hop_by_hop_headers(headers: &mut HashMap<String, String>) {
    let hop_by_hop = [
        "proxy-connection",
        "proxy-authenticate",
        "proxy-authorization",
        "te",
        "trailers",
        "transfer-encoding",
        "upgrade",
        "connection",
    ];

    for h in hop_by_hop {
        headers.retain(|k, _| !k.eq_ignore_ascii_case(h));
    }
}

pub fn parse_host(raw_host: &str, default_port: u16) -> Result<Destination> {
    let raw = raw_host.trim();
    if let Some((h, p)) = raw.rsplit_once(':')
        && let Ok(port) = p.parse::<u16>()
    {
        let addr = Address::from_str(h)?;
        return Ok(Destination::tcp(addr, port));
    }
    let addr = Address::from_str(raw)?;
    Ok(Destination::tcp(addr, default_port))
}
