// Module: main\commands\all\x25519.rs
// 1:1 Rust implementation corresponding to Go main\commands\all\x25519.go

use super::curve25519::gen_curve25519;
use crate::main::commands::base::command::Command;
use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};

pub fn cmd_x25519() -> Command {
    Command::new(
        "x25519",
        "xray x25519 [-i key] [--std-encoding]",
        "Generate key pair for X25519 key exchange (REALITY, VLESS Encryption)",
    )
    .with_run(|args| {
        let mut input_key = None;
        let mut std_encoding = false;

        let mut i = 0;
        while i < args.len() {
            if args[i] == "--std-encoding" {
                std_encoding = true;
            } else if args[i] == "-i" && i + 1 < args.len() {
                let decoded = if std_encoding {
                    STANDARD.decode(args[i + 1])
                } else {
                    URL_SAFE_NO_PAD.decode(args[i + 1])
                };
                if let Ok(key) = decoded {
                    input_key = Some(key);
                }
                i += 1;
            }
            i += 1;
        }

        let (priv_k, pub_k, hash) = gen_curve25519(input_key.as_deref())?;
        let encode = |b: &[u8]| {
            if std_encoding {
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
    fn test_x25519_command() {
        let cmd = cmd_x25519();
        let res = cmd.execute(&[]).unwrap();
        assert!(res.contains("PrivateKey:"));
        assert!(res.contains("Password (PublicKey):"));

        let std_res = cmd.execute(&["--std-encoding"]).unwrap();
        assert!(std_res.contains("PrivateKey:"));
    }
}
