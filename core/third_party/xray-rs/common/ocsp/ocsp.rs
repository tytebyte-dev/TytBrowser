// Module: common\ocsp\ocsp.rs
// 1:1 Rust implementation corresponding to Go common\ocsp\ocsp.go

#[derive(Debug, Clone, Default)]
pub struct OcspCache {
    pub response: Vec<u8>,
}

impl OcspCache {
    pub fn new(response: Vec<u8>) -> Self {
        Self { response }
    }

    pub fn is_valid(&self) -> bool {
        !self.response.is_empty()
    }
}
