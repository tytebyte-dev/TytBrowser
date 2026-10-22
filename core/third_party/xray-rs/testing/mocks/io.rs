// Module: testing\mocks\io.rs
// Mock IO stream with memory buffer

use tokio::io::DuplexStream;

pub struct MockIoPair {
    pub client: DuplexStream,
    pub server: DuplexStream,
}

impl MockIoPair {
    pub fn new(buf_size: usize) -> Self {
        let (c, s) = tokio::io::duplex(buf_size);
        Self {
            client: c,
            server: s,
        }
    }
}
