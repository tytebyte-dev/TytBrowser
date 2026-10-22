// Module: main\commands\all\wg.rs
// 1:1 Rust implementation corresponding to Go main\commands\all\wg.go

use super::curve25519::gen_curve25519;
use crate::main::commands::base::command::Command;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;

pub fn cmd_wg() -> Command {
    Command::new(
        "wg",
        "xray wg [-i key]",
        "Generate key pair for X25519 key exchange (WireGuard)",
    )
    .with_run(|args| {
        let mut input_key = None;
        let mut i = 0;
        while i < args.len() {
            if args[i] == "-i" && i + 1 < args.len() {
                if let Ok(key) = STANDARD.decode(args[i + 1]) {
                    input_key = Some(key);
                }
                i += 1;
            }
            i += 1;
        }

        let (priv_k, pub_k, hash) = gen_curve25519(input_key.as_deref())?;
        Ok(format!(
            "PrivateKey: {}\nPassword (PublicKey): {}\nHash32: {}",
            STANDARD.encode(priv_k),
            STANDARD.encode(pub_k),
            STANDARD.encode(hash)
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wg_command() {
        let cmd = cmd_wg();
        let res = cmd.execute(&[]).unwrap();
        assert!(res.contains("PrivateKey:"));
        assert!(res.contains("Password (PublicKey):"));
    }
}
