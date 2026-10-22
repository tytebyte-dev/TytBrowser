// Module: transport\internet\memory_settings.rs
// 1:1 Rust implementation corresponding to Go transport\internet\memory_settings.go

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemorySettings {
    pub buffer_size: usize,
    pub max_streams: usize,
}

impl Default for MemorySettings {
    fn default() -> Self {
        Self {
            buffer_size: 512 * 1024,
            max_streams: 1024,
        }
    }
}

use super::sockopt::SocketOptions;

#[derive(Debug, Clone, Default)]
pub struct MemoryStreamConfig {
    pub protocol_name: String,
    pub socket_settings: Option<SocketOptions>,
}
