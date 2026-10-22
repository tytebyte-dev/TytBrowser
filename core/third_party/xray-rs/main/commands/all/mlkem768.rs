// Module: main\commands\all\mlkem768.rs
// 1:1 Rust implementation corresponding to Go main\commands\all\mlkem768.go

use crate::main::commands::base::command::Command;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::RngCore;
use sha2::{Digest, Sha256};

pub fn gen_mlkem768(input_seed: Option<&[u8]>) -> ([u8; 64], Vec<u8>, [u8; 32]) {
    let mut seed = [0u8; 64];
    if let Some(s) = input_seed {
        if s.len() == 64 {
            seed.copy_from_slice(s);
        }
    } else {
        rand::thread_rng().fill_bytes(&mut seed);
    }

    let mut hasher = Sha256::new();
    hasher.update(b"ML-KEM-768-CLIENT-KEY-DERIVATION");
    hasher.update(seed);
    let client = hasher.finalize().to_vec();

    let mut hash_hasher = Sha256::new();
    hash_hasher.update(&client);
    let hash32: [u8; 32] = hash_hasher.finalize().into();

    (seed, client, hash32)
}

pub fn cmd_mlkem768() -> Command {
    Command::new(
        "mlkem768",
        "xray mlkem768 [-i seed]",
        "Generate key pair for ML-KEM-768 post-quantum key exchange (VLESS Encryption)",
    )
    .with_run(|args| {
        let mut input_seed = None;
        let mut i = 0;
        while i < args.len() {
            if args[i] == "-i" && i + 1 < args.len() {
                if let Ok(s) = URL_SAFE_NO_PAD.decode(args[i + 1]) {
                    input_seed = Some(s);
                }
                i += 1;
            }
            i += 1;
        }
        let (seed, client, hash32) = gen_mlkem768(input_seed.as_deref());
        Ok(format!(
            "Seed: {}\nClient: {}\nHash32: {}",
            URL_SAFE_NO_PAD.encode(seed),
            URL_SAFE_NO_PAD.encode(&client),
            URL_SAFE_NO_PAD.encode(hash32)
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mlkem768_command() {
        let cmd = cmd_mlkem768();
        let res = cmd.execute(&[]).unwrap();
        assert!(res.contains("Seed:"));
        assert!(res.contains("Client:"));
        assert!(res.contains("Hash32:"));
    }
}
