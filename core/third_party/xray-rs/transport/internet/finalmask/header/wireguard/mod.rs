pub mod config;
pub mod conn;

pub use conn::WireguardPacketConn;

#[derive(Debug, Clone)]
pub struct WireguardHeader;

impl WireguardHeader {
    pub fn new() -> Self {
        Self
    }

    pub fn size(&self) -> usize {
        4
    }

    pub fn serialize(&self, buf: &mut [u8]) {
        buf[0] = 0x04;
        buf[1] = 0x00;
        buf[2] = 0x00;
        buf[3] = 0x00;
    }

    pub fn write_header(&self) -> [u8; 4] {
        [0x04, 0x00, 0x00, 0x00]
    }
}

impl Default for WireguardHeader {
    fn default() -> Self {
        Self::new()
    }
}
