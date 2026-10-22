pub mod headers;
pub mod sniff;

#[cfg(test)]
pub mod sniff_test;

pub use headers::{parse_host, parse_x_forwarded_for, remove_hop_by_hop_headers};
pub use sniff::{HttpVersion, SniffHeader, sniff_http};
