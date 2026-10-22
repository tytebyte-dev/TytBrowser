// Module: transport\internet\finalmask\salamander\salamander.rs
// 1:1 Rust implementation corresponding to Go transport\internet\finalmask\salamander\salamander.go

use crate::common::errors::{Error, Result};
use rand::Rng;

pub const SM_PSK_MIN_LEN: usize = 4;
pub const SM_SALT_LEN: usize = 8;
pub const SM_KEY_LEN: usize = 32;

const IV: [u64; 8] = [
    0x6a09e667f3bcc908,
    0xbb67ae8584caa73b,
    0x3c6ef372fe94f82b,
    0xa54ff53a5f1d36f1,
    0x510e527fade682d1,
    0x9b05688c2b3e6c1f,
    0x1f83d9abfb41bd6b,
    0x5be0cd19137e2179,
];

const SIGMA: [[usize; 16]; 10] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3],
    [11, 8, 12, 0, 5, 2, 15, 13, 10, 14, 3, 6, 7, 1, 9, 4],
    [7, 9, 3, 1, 13, 12, 11, 14, 2, 6, 5, 10, 4, 0, 15, 8],
    [9, 0, 5, 7, 2, 4, 10, 15, 14, 1, 11, 12, 6, 8, 3, 13],
    [2, 12, 6, 10, 0, 11, 8, 3, 4, 13, 7, 5, 15, 14, 1, 9],
    [12, 5, 1, 15, 14, 13, 4, 10, 0, 7, 6, 3, 9, 2, 8, 11],
    [13, 11, 7, 14, 12, 1, 3, 9, 5, 0, 15, 4, 8, 6, 2, 10],
    [6, 15, 14, 9, 11, 3, 0, 8, 12, 2, 13, 7, 1, 4, 10, 5],
    [10, 2, 8, 4, 7, 6, 1, 5, 15, 11, 9, 14, 3, 12, 13, 0],
];

#[inline]
fn g(v: &mut [u64; 16], a: usize, b: usize, c: usize, d: usize, x: u64, y: u64) {
    v[a] = v[a].wrapping_add(v[b]).wrapping_add(x);
    v[d] = (v[d] ^ v[a]).rotate_right(32);
    v[c] = v[c].wrapping_add(v[d]);
    v[b] = (v[b] ^ v[c]).rotate_right(24);
    v[a] = v[a].wrapping_add(v[b]).wrapping_add(y);
    v[d] = (v[d] ^ v[a]).rotate_right(16);
    v[c] = v[c].wrapping_add(v[d]);
    v[b] = (v[b] ^ v[c]).rotate_right(63);
}

fn blake2b_compress(h: &mut [u64; 8], block: &[u8; 128], t: u128, last: bool) {
    let mut v = [0u64; 16];
    v[..8].copy_from_slice(h);
    v[8..].copy_from_slice(&IV);
    v[12] ^= t as u64;
    v[13] ^= (t >> 64) as u64;
    if last {
        v[14] ^= !0u64;
    }

    let mut m = [0u64; 16];
    for i in 0..16 {
        m[i] = u64::from_le_bytes(block[i * 8..(i + 1) * 8].try_into().unwrap());
    }

    for r in 0..12 {
        let s = &SIGMA[r % 10];
        g(&mut v, 0, 4, 8, 12, m[s[0]], m[s[1]]);
        g(&mut v, 1, 5, 9, 13, m[s[2]], m[s[3]]);
        g(&mut v, 2, 6, 10, 14, m[s[4]], m[s[5]]);
        g(&mut v, 3, 7, 11, 15, m[s[6]], m[s[7]]);
        g(&mut v, 0, 5, 10, 15, m[s[8]], m[s[9]]);
        g(&mut v, 1, 6, 11, 12, m[s[10]], m[s[11]]);
        g(&mut v, 2, 7, 8, 13, m[s[12]], m[s[13]]);
        g(&mut v, 3, 4, 9, 14, m[s[14]], m[s[15]]);
    }

    for i in 0..8 {
        h[i] ^= v[i] ^ v[i + 8];
    }
}

pub fn blake2b_256(key: &[u8], data: &[u8]) -> [u8; 32] {
    let mut h = IV;
    let k_len = key.len();
    h[0] ^= 0x01010000 ^ ((k_len as u64) << 8) ^ 32u64;

    let mut buf = [0u8; 128];
    let mut buf_len = 0;
    let mut total_bytes: u128 = 0;

    if k_len > 0 {
        buf[..k_len].copy_from_slice(key);
        buf_len = 128;
    }

    let mut offset = 0;
    while offset < data.len() {
        if buf_len == 128 {
            total_bytes += 128;
            blake2b_compress(&mut h, &buf, total_bytes, false);
            buf = [0u8; 128];
            buf_len = 0;
        }
        let take = (128 - buf_len).min(data.len() - offset);
        buf[buf_len..buf_len + take].copy_from_slice(&data[offset..offset + take]);
        buf_len += take;
        offset += take;
    }

    total_bytes += buf_len as u128;
    blake2b_compress(&mut h, &buf, total_bytes, true);

    let mut out = [0u8; 32];
    for i in 0..4 {
        out[i * 8..(i + 1) * 8].copy_from_slice(&h[i].to_le_bytes());
    }
    out
}

pub struct SalamanderObfuscator {
    psk: Vec<u8>,
}

impl SalamanderObfuscator {
    pub fn new(psk: Vec<u8>) -> Result<Self> {
        if psk.len() < SM_PSK_MIN_LEN {
            return Err(Error::Config(format!(
                "Salamander PSK must be at least {} bytes",
                SM_PSK_MIN_LEN
            )));
        }
        Ok(Self { psk })
    }

    pub fn key(&self, salt: &[u8]) -> [u8; SM_KEY_LEN] {
        blake2b_256(&self.psk, salt)
    }

    pub fn obfuscate(&self, input: &[u8]) -> Vec<u8> {
        let mut out = vec![0u8; input.len() + SM_SALT_LEN];
        self.obfuscate_slice(input, &mut out);
        out
    }

    pub fn deobfuscate(&self, input: &[u8]) -> Result<Vec<u8>> {
        if input.len() <= SM_SALT_LEN {
            return Err(Error::Protocol("Salamander packet too short".into()));
        }
        let mut out = vec![0u8; input.len() - SM_SALT_LEN];
        let n = self.deobfuscate_slice(input, &mut out);
        if n == 0 {
            return Err(Error::Protocol("Salamander deobfuscate failed".into()));
        }
        Ok(out)
    }

    pub fn obfuscate_slice(&self, input: &[u8], output: &mut [u8]) -> usize {
        let out_len = input.len() + SM_SALT_LEN;
        if output.len() < out_len {
            return 0;
        }

        let mut salt = [0u8; SM_SALT_LEN];
        rand::thread_rng().fill(&mut salt);
        output[..SM_SALT_LEN].copy_from_slice(&salt);

        let key = self.key(&salt);
        for (i, &b) in input.iter().enumerate() {
            output[i + SM_SALT_LEN] = b ^ key[i % SM_KEY_LEN];
        }
        out_len
    }

    pub fn deobfuscate_slice(&self, input: &[u8], output: &mut [u8]) -> usize {
        if input.len() <= SM_SALT_LEN {
            return 0;
        }
        let out_len = input.len() - SM_SALT_LEN;
        if output.len() < out_len {
            return 0;
        }

        let salt = &input[..SM_SALT_LEN];
        let key = self.key(salt);
        for (i, &b) in input[SM_SALT_LEN..].iter().enumerate() {
            output[i] = b ^ key[i % SM_KEY_LEN];
        }
        out_len
    }
}
