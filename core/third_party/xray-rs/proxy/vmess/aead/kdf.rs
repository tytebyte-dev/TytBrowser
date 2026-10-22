// Module: proxy\vmess\aead\kdf.rs
// 1:1 Rust implementation corresponding to Go proxy\vmess\aead\kdf.go

use hkdf::Hkdf;
use sha2::Sha256;

pub fn vmess_kdf_1(key: &[u8], path1: &[u8]) -> [u8; 16] {
    let hk = Hkdf::<Sha256>::new(Some(path1), key);
    let mut out = [0u8; 16];
    hk.expand(b"", &mut out).unwrap();
    out
}

pub fn vmess_kdf_2(key: &[u8], path1: &[u8], path2: &[u8]) -> [u8; 16] {
    let hk = Hkdf::<Sha256>::new(Some(path1), key);
    let mut inter = [0u8; 32];
    hk.expand(path2, &mut inter).unwrap();
    let hk2 = Hkdf::<Sha256>::new(Some(path2), &inter);
    let mut out = [0u8; 16];
    hk2.expand(b"", &mut out).unwrap();
    out
}
