// Module: transport\internet\finalmask\mkcp\original\xor.rs
// 1:1 Rust implementation corresponding to Go transport\internet\finalmask\mkcp\original\xor.go

/// xorfwd performs XOR forwards in words, x[i] ^= x[i-4], i from 4 to len
pub fn xorfwd(x: &mut [u8]) {
    for i in 4..x.len() {
        x[i] ^= x[i - 4];
    }
}

/// xorbkd performs XOR backwards in words, x[i] ^= x[i-4], i from len - 1 to 4
pub fn xorbkd(x: &mut [u8]) {
    for i in (4..x.len()).rev() {
        x[i] ^= x[i - 4];
    }
}
