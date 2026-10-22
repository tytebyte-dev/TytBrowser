// Module: transport\internet\splithttp\xpadding.rs
// 1:1 Rust implementation corresponding to Go transport\internet\splithttp\xpadding.go

use rand::Rng;

pub const PADDING_METHOD_REPEAT_X: &str = "repeat-x";
pub const PADDING_METHOD_TOKENISH: &str = "tokenish";

pub const CHARSET_BASE62: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

pub fn rand_string_base62(n: usize) -> String {
    if n == 0 {
        return String::new();
    }
    let mut rng = rand::thread_rng();
    let mut s = String::with_capacity(n);
    for _ in 0..n {
        let idx = rng.gen_range(0..CHARSET_BASE62.len());
        s.push(CHARSET_BASE62[idx] as char);
    }
    s
}

pub fn generate_tokenish_padding(target_bytes: usize) -> String {
    if target_bytes == 0 {
        return String::new();
    }
    rand_string_base62(target_bytes)
}

pub fn generate_repeat_x(target_bytes: usize) -> String {
    if target_bytes == 0 {
        return String::new();
    }
    "X".repeat(target_bytes)
}

pub fn generate_padding(method: &str, length: usize) -> String {
    match method {
        PADDING_METHOD_REPEAT_X => generate_repeat_x(length),
        _ => generate_tokenish_padding(length),
    }
}

pub fn apply_x_padding(
    headers: &mut Vec<(String, String)>,
    header_key: &str,
    method: &str,
    length: usize,
) {
    if length > 0 {
        let padding = generate_padding(method, length);
        headers.push((header_key.to_string(), padding));
    }
}

pub struct XPadding;

impl XPadding {
    pub fn generate(method: &str, length: usize) -> String {
        generate_padding(method, length)
    }
}
