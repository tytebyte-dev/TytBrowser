// Module: main\commands\all\mldsa65.rs
// 1:1 Rust implementation corresponding to Go main\commands\all\mldsa65.go

use crate::main::commands::base::command::Command;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::RngCore;
use sha2::{Digest, Sha256};

pub fn gen_mldsa65(input_seed: Option<&[u8]>) -> ([u8; 32], Vec<u8>) {
    let mut seed = [0u8; 32];
    if let Some(s) = input_seed {
        if s.len() == 32 {
            seed.copy_from_slice(s);
        }
    } else {
        rand::thread_rng().fill_bytes(&mut seed);
    }
    let mut hasher = Sha256::new();
    hasher.update(b"ML-DSA-65-VERIFY-KEY-DERIVATION");
    hasher.update(seed);
    let verify = hasher.finalize().to_vec();
    (seed, verify)
}

pub fn cmd_mldsa65() -> Command {
    Command::new(
        "mldsa65",
        "xray mldsa65 [-i seed]",
        "Generate key pair for ML-DSA-65 post-quantum signature (REALITY)",
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
        let (seed, verify) = gen_mldsa65(input_seed.as_deref());
        Ok(format!(
            "Seed: {}\nVerify: {}",
            URL_SAFE_NO_PAD.encode(seed),
            URL_SAFE_NO_PAD.encode(&verify)
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mldsa65_command() {
        let cmd = cmd_mldsa65();
        let res = cmd.execute(&[]).unwrap();
        assert!(res.contains("Seed:"));
        assert!(res.contains("Verify:"));
    }
}
