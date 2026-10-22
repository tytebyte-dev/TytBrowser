// Module: main\commands\all\curve25519.rs
// 1:1 Rust implementation corresponding to Go main\commands\all\curve25519.go

use crate::common::errors::Result;
use crate::main::commands::base::command::Command;
use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use rand::RngCore;
use sha2::{Digest, Sha256};
use x25519_dalek::{PublicKey, StaticSecret};

pub fn gen_curve25519(input_key: Option<&[u8]>) -> Result<([u8; 32], [u8; 32], [u8; 32])> {
    let mut priv_bytes = [0u8; 32];
    if let Some(input) = input_key {
        if input.len() == 32 {
            priv_bytes.copy_from_slice(input);
        } else {
            return Err(crate::common::errors::Error::Config(
                "Invalid length of X25519 private key".into(),
            ));
        }
    } else {
        rand::thread_rng().fill_bytes(&mut priv_bytes);
    }

    priv_bytes[0] &= 248;
    priv_bytes[31] &= 127;
    priv_bytes[31] |= 64;

    let secret = StaticSecret::from(priv_bytes);
    let public = PublicKey::from(&secret);
    let pub_bytes = *public.as_bytes();

    let mut hasher = Sha256::new();
    hasher.update(pub_bytes);
    let hash: [u8; 32] = hasher.finalize().into();

    Ok((priv_bytes, pub_bytes, hash))
}

pub fn cmd_curve25519() -> Command {
    Command::new(
        "curve25519",
        "xray curve25519 [-i key] [-std]",
        "Generate Curve25519 keypair and hash",
    )
    .with_run(|args| {
        let (priv_k, pub_k, hash) = gen_curve25519(None)?;
        let std_enc = args.contains(&"-std");
        let encode = |b: &[u8]| {
            if std_enc {
                STANDARD.encode(b)
            } else {
                URL_SAFE_NO_PAD.encode(b)
            }
        };
        Ok(format!(
            "PrivateKey: {}\nPassword (PublicKey): {}\nHash32: {}",
            encode(&priv_k),
            encode(&pub_k),
            encode(&hash)
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gen_curve25519() {
        let (priv_k, pub_k, hash) = gen_curve25519(None).unwrap();
        assert_eq!(priv_k.len(), 32);
        assert_eq!(pub_k.len(), 32);
        assert_eq!(hash.len(), 32);

        // Clamping checks
        assert_eq!(priv_k[0] & 7, 0);
        assert_eq!(priv_k[31] & 128, 0);
        assert_ne!(priv_k[31] & 64, 0);

        // Deterministic generation from input key
        let (p2, pub2, h2) = gen_curve25519(Some(&priv_k)).unwrap();
        assert_eq!(priv_k, p2);
        assert_eq!(pub_k, pub2);
        assert_eq!(hash, h2);

        let cmd = cmd_curve25519();
        let out = cmd.execute(&[]).unwrap();
        assert!(out.contains("PrivateKey:"));
        assert!(out.contains("Password (PublicKey):"));
    }
}
