// Module: transport\internet\headers\http\resp.rs
// 1:1 Rust implementation corresponding to Go transport\internet\headers\http\resp.go

use super::config_pb::{Header, ResponseConfig, Status, Version};

pub fn resp400() -> ResponseConfig {
    ResponseConfig {
        version: Some(Version {
            value: "1.1".into(),
        }),
        status: Some(Status {
            code: "400".into(),
            reason: "Bad Request".into(),
        }),
        header: vec![
            Header {
                name: "Connection".into(),
                value: vec!["close".into()],
            },
            Header {
                name: "Cache-Control".into(),
                value: vec!["private".into()],
            },
            Header {
                name: "Content-Length".into(),
                value: vec!["0".into()],
            },
        ],
    }
}

pub fn resp404() -> ResponseConfig {
    ResponseConfig {
        version: Some(Version {
            value: "1.1".into(),
        }),
        status: Some(Status {
            code: "404".into(),
            reason: "Not Found".into(),
        }),
        header: vec![
            Header {
                name: "Connection".into(),
                value: vec!["close".into()],
            },
            Header {
                name: "Cache-Control".into(),
                value: vec!["private".into()],
            },
            Header {
                name: "Content-Length".into(),
                value: vec!["0".into()],
            },
        ],
    }
}
