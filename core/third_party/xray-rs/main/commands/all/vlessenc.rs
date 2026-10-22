// Module: main\commands\all\vlessenc.rs
// 1:1 Rust implementation corresponding to Go main\commands\all\vlessenc.go

use super::curve25519::gen_curve25519;
use super::mlkem768::gen_mlkem768;
use crate::main::commands::base::command::Command;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

pub fn generate_dot_config(fields: &[&str]) -> String {
    fields.join(".")
}

pub fn cmd_vlessenc() -> Command {
    Command::new(
        "vlessenc",
        "xray vlessenc",
        "Generate decryption/encryption json pair (VLESS Encryption)",
    )
    .with_run(|_args| {
        let (private_key, password, _) = gen_curve25519(None)?;
        let server_key = URL_SAFE_NO_PAD.encode(private_key);
        let client_key = URL_SAFE_NO_PAD.encode(password);

        let decryption = generate_dot_config(&["mlkem768x25519plus", "native", "600s", &server_key]);
        let encryption = generate_dot_config(&["mlkem768x25519plus", "native", "0rtt", &client_key]);

        let (seed, client, _) = gen_mlkem768(None);
        let server_key_pq = URL_SAFE_NO_PAD.encode(seed);
        let client_key_pq = URL_SAFE_NO_PAD.encode(&client);

        let decryption_pq = generate_dot_config(&["mlkem768x25519plus", "native", "600s", &server_key_pq]);
        let encryption_pq = generate_dot_config(&["mlkem768x25519plus", "native", "0rtt", &client_key_pq]);

        Ok(format!(
            "Choose one Authentication to use, do not mix them. Ephemeral key exchange is Post-Quantum safe anyway.\n\n\
            Authentication: X25519, not Post-Quantum\n\"decryption\": \"{}\"\n\"encryption\": \"{}\"\n\n\
            Authentication: ML-KEM-768, Post-Quantum\n\"decryption\": \"{}\"\n\"encryption\": \"{}\"",
            decryption, encryption, decryption_pq, encryption_pq
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vlessenc_command() {
        let cmd = cmd_vlessenc();
        let out = cmd.execute(&[]).unwrap();
        assert!(out.contains("Authentication: X25519"));
        assert!(out.contains("Authentication: ML-KEM-768"));
        assert!(out.contains("mlkem768x25519plus.native"));
    }
}
