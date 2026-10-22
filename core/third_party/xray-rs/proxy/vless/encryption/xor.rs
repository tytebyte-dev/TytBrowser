// Module: proxy\vless\encryption\xor.rs
// 1:1 Rust implementation corresponding to Go proxy\vless\encryption\xor.go

pub fn xor_inplace(data: &mut [u8], key: &[u8]) {
    if key.is_empty() {
        return;
    }
    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= key[i % key.len()];
    }
}
