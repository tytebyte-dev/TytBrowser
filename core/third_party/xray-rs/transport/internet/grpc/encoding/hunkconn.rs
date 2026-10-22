// Module: transport\internet\grpc\encoding\hunkconn.rs
// 1:1 Rust implementation corresponding to Go transport\internet\grpc\encoding\hunkconn.go

pub struct HunkConnection {
    pub buffer: Vec<u8>,
}

impl HunkConnection {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }
}

impl Default for HunkConnection {
    fn default() -> Self {
        Self::new()
    }
}
