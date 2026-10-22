pub mod config;
pub mod conn;

use rand::Rng;

#[derive(Debug, Clone)]
pub struct WeChatHeader {
    pub sn: u32,
}

impl WeChatHeader {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            sn: rng.r#gen::<u16>() as u32,
        }
    }

    pub fn size(&self) -> usize {
        13
    }

    pub fn serialize(&mut self, buf: &mut [u8]) {
        self.sn = self.sn.wrapping_add(1);
        buf[0] = 0xa1;
        buf[1] = 0x08;
        buf[2..6].copy_from_slice(&self.sn.to_be_bytes());
        buf[6] = 0x00;
        buf[7] = 0x10;
        buf[8] = 0x11;
        buf[9] = 0x18;
        buf[10] = 0x30;
        buf[11] = 0x22;
        buf[12] = 0x30;
    }

    pub fn write_header(&mut self) -> [u8; 13] {
        let mut b = [0u8; 13];
        self.serialize(&mut b);
        b
    }
}

impl Default for WeChatHeader {
    fn default() -> Self {
        Self::new()
    }
}
