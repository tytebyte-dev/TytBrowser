// Module: common\utils\padding.rs
// 1:1 Rust implementation corresponding to Go common\utils\padding.go

use rand::Rng;

pub fn generate_random_padding(min_len: usize, max_len: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let len = rng.gen_range(min_len..=max_len);
    let mut buf = vec![0u8; len];
    rng.fill(&mut buf[..]);
    buf
}
