// Module: main\commands\all\tls\ech.rs
// 1:1 Rust implementation corresponding to Go main\commands\all\tls\ech.go

use crate::main::commands::base::command::Command;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::RngCore;

pub fn generate_ech_keypair() -> ([u8; 32], Vec<u8>) {
    let mut priv_key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut priv_key);
    let mut config = Vec::new();
    config.extend_from_slice(b"\xfe\x0d"); // ECH draft version
    config.extend_from_slice(&priv_key);
    (priv_key, config)
}

pub fn cmd_ech() -> Command {
    Command::new(
        "ech",
        "xray tls ech",
        "Generate ECH (Encrypted Client Hello) keys",
    )
    .with_run(|_args| {
        let (priv_k, ech_config) = generate_ech_keypair();
        Ok(format!(
            "ECH Private Key: {}\nECH Config: {}",
            URL_SAFE_NO_PAD.encode(priv_k),
            URL_SAFE_NO_PAD.encode(&ech_config)
        ))
    })
}
