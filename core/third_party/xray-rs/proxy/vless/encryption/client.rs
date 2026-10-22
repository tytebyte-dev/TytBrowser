// Module: proxy\vless\encryption\client.rs
// 1:1 Rust implementation corresponding to Go proxy\vless\encryption\client.go

use super::xor::xor_inplace;

pub struct VlessXorClient {
    key: Vec<u8>,
}

impl VlessXorClient {
    pub fn new(key: Vec<u8>) -> Self {
        Self { key }
    }

    pub fn encrypt(&self, data: &mut [u8]) {
        xor_inplace(data, &self.key);
    }
}
