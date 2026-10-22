// Module: transport\internet\headers\http\mod.rs
// 1:1 Rust implementation corresponding to Go transport\internet\headers\http

pub mod config;
#[path = "config.pb.rs"]
pub mod config_pb;
pub mod http;
#[path = "linkedreadRequest.rs"]
pub mod linked_read_request;
pub mod resp;

#[cfg(test)]
pub mod http_test;

pub use config_pb::{Config, Header, Method, RequestConfig, ResponseConfig, Status, Version};
pub use http::{
    Authenticator, CRLF, ENDING, HeaderReader, HeaderWriter, HttpHeaderObfuscator, HttpStream,
    MAX_HEADER_LENGTH, new_authenticator,
};
pub use resp::{resp400, resp404};
