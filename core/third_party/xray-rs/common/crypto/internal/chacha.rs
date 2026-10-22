// Module: common\crypto\internal\chacha.rs
// 1:1 Rust implementation corresponding to Go common\crypto\internal\chacha.go

use crate::common::crypto::crypto::StreamCipher;
use crate::common::errors::Result;

pub struct ChaChaCore {
    pub state: [u32; 16],
}

#[inline(always)]
fn qr(x: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
    x[a] = x[a].wrapping_add(x[b]);
    x[d] ^= x[a];
    x[d] = x[d].rotate_left(16);

    x[c] = x[c].wrapping_add(x[d]);
    x[b] ^= x[c];
    x[b] = x[b].rotate_left(12);

    x[a] = x[a].wrapping_add(x[b]);
    x[d] ^= x[a];
    x[d] = x[d].rotate_left(8);

    x[c] = x[c].wrapping_add(x[d]);
    x[b] ^= x[c];
    x[b] = x[b].rotate_left(7);
}

impl ChaChaCore {
    pub fn new(key: &[u8; 32], nonce: &[u8; 12]) -> Self {
        let mut state = [0u32; 16];
        state[0] = 0x61707865;
        state[1] = 0x3320646e;
        state[2] = 0x79622d32;
        state[3] = 0x6b206574;
        for i in 0..8 {
            state[4 + i] = u32::from_le_bytes(key[i * 4..(i + 1) * 4].try_into().unwrap());
        }
        state[12] = 0; // counter
        for i in 0..3 {
            state[13 + i] = u32::from_le_bytes(nonce[i * 4..(i + 1) * 4].try_into().unwrap());
        }
        Self { state }
    }

    pub fn set_counter(&mut self, counter: u32) {
        self.state[12] = counter;
    }

    pub fn block(&mut self, output: &mut [u8; 64]) {
        let mut x = self.state;
        for _ in 0..10 {
            qr(&mut x, 0, 4, 8, 12);
            qr(&mut x, 1, 5, 9, 13);
            qr(&mut x, 2, 6, 10, 14);
            qr(&mut x, 3, 7, 11, 15);

            qr(&mut x, 0, 5, 10, 15);
            qr(&mut x, 1, 6, 11, 12);
            qr(&mut x, 2, 7, 8, 13);
            qr(&mut x, 3, 4, 9, 14);
        }

        for i in 0..16 {
            let val = x[i].wrapping_add(self.state[i]);
            output[i * 4..(i + 1) * 4].copy_from_slice(&val.to_le_bytes());
        }
        self.state[12] = self.state[12].wrapping_add(1);
    }
}

impl StreamCipher for ChaChaCore {
    fn encrypt(&mut self, buffer: &mut [u8]) -> Result<()> {
        let mut block = [0u8; 64];
        let mut offset = 0;
        while offset < buffer.len() {
            self.block(&mut block);
            let chunk = std::cmp::min(64, buffer.len() - offset);
            for i in 0..chunk {
                buffer[offset + i] ^= block[i];
            }
            offset += chunk;
        }
        Ok(())
    }

    fn decrypt(&mut self, buffer: &mut [u8]) -> Result<()> {
        self.encrypt(buffer)
    }
}
