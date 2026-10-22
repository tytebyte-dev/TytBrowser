pub mod config;
pub mod conn;

pub use conn::UtpPacketConn;

use rand::Rng;

#[derive(Debug, Clone)]
pub struct UtpHeader {
    pub header: u8,
    pub extension: u8,
    pub connection_id: u16,
}

impl UtpHeader {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            header: 1,
            extension: 0,
            connection_id: rng.r#gen(),
        }
    }

    pub fn size(&self) -> usize {
        4
    }

    pub fn serialize(&self, buf: &mut [u8]) {
        buf[0..2].copy_from_slice(&self.connection_id.to_be_bytes());
        buf[2] = self.header;
        buf[3] = self.extension;
    }

    pub fn write_header(&self) -> [u8; 4] {
        let mut b = [0u8; 4];
        self.serialize(&mut b);
        b
    }
}

impl Default for UtpHeader {
    fn default() -> Self {
        Self::new()
    }
}
