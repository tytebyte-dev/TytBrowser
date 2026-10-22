// Module: proxy\dokodemo\dokodemo.rs
// 1:1 Rust implementation corresponding to Go proxy\dokodemo\dokodemo.go

use super::config::DokodemoConfig;
use crate::common::net::Destination;

pub struct DokodemoHandler {
    config: DokodemoConfig,
}

impl DokodemoHandler {
    pub fn new(config: DokodemoConfig) -> Self {
        Self { config }
    }

    pub fn destination(&self) -> Destination {
        Destination::tcp(self.config.address.clone(), self.config.port)
    }
}
