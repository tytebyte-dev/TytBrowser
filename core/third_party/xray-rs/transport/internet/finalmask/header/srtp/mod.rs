pub mod config;
pub mod conn;

pub use conn::SrtpPacketConn;

use rand::Rng;

#[derive(Debug, Clone)]
pub struct SrtpHeader {
    pub header: u16,
    pub number: u16,
}

impl SrtpHeader {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            header: 0xB5E8,
            number: rng.r#gen(),
        }
    }

    pub fn size(&self) -> usize {
        4
    }

    pub fn serialize(&mut self, buf: &mut [u8]) {
        self.number = self.number.wrapping_add(1);
        buf[0..2].copy_from_slice(&self.header.to_be_bytes());
        buf[2..4].copy_from_slice(&self.number.to_be_bytes());
    }

    pub fn write_header(&mut self) -> [u8; 4] {
        let mut b = [0u8; 4];
        self.serialize(&mut b);
        b
    }
}

impl Default for SrtpHeader {
    fn default() -> Self {
        Self::new()
    }
}
