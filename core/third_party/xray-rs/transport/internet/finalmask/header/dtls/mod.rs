pub mod config;
pub mod conn;

use rand::Rng;

#[derive(Debug, Clone)]
pub struct DtlsHeader {
    pub epoch: u16,
    pub sequence: u32,
    pub length: u16,
}

impl DtlsHeader {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            epoch: rng.r#gen(),
            sequence: 0,
            length: 17,
        }
    }

    pub fn size(&self) -> usize {
        13
    }

    pub fn serialize(&mut self, buf: &mut [u8]) {
        buf[0] = 23; // ContentType: Application Data
        buf[1] = 254; // DTLS 1.2 major version
        buf[2] = 253; // DTLS 1.2 minor version
        buf[3..5].copy_from_slice(&self.epoch.to_be_bytes());
        buf[5] = 0;
        buf[6] = 0;
        buf[7..11].copy_from_slice(&self.sequence.to_be_bytes());
        self.sequence = self.sequence.wrapping_add(1);
        buf[11..13].copy_from_slice(&self.length.to_be_bytes());
        self.length = self.length.wrapping_add(17);
        if self.length > 100 {
            self.length = self.length.saturating_sub(50);
        }
    }

    pub fn write_header(&mut self) -> [u8; 13] {
        let mut b = [0u8; 13];
        self.serialize(&mut b);
        b
    }
}

impl Default for DtlsHeader {
    fn default() -> Self {
        Self::new()
    }
}
