// Module: main\commands\all\tls\cert.rs
// 1:1 Rust implementation corresponding to Go main\commands\all\tls\cert.go

use crate::common::errors::{Error, Result};
use crate::main::commands::base::command::Command;

pub fn generate_self_signed_cert(domain: &str) -> Result<(String, String)> {
    let subject_alt_names = vec![domain.to_string()];
    let cert = rcgen::generate_simple_self_signed(subject_alt_names)
        .map_err(|e| Error::Config(format!("Failed to generate self-signed cert: {}", e)))?;
    let cert_pem = cert.cert.pem();
    let key_pem = cert.key_pair.serialize_pem();
    Ok((cert_pem, key_pem))
}

pub fn cmd_cert() -> Command {
    Command::new(
        "cert",
        "xray tls cert [domain]",
        "Generate self-signed TLS certificates",
    )
    .with_run(|args| {
        let domain = args.first().copied().unwrap_or("example.com");
        let (cert, key) = generate_self_signed_cert(domain)?;
        Ok(format!("Certificate:\n{}\nKey:\n{}", cert, key))
    })
}
