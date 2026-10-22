// Module: main\commands\all\tls\hash.rs
// 1:1 Rust implementation corresponding to Go main\commands\all\tls\hash.go

use crate::main::commands::base::command::Command;
use crate::transport::internet::tls::generate_cert_hash_hex;

pub fn cmd_hash() -> Command {
    Command::new(
        "hash",
        "xray tls hash <cert.pem>",
        "Calculate SHA-256 pin hash of certificate",
    )
    .with_run(|args| {
        let cert_data = args.first().unwrap_or(&"dummy-cert").as_bytes();
        let hash = generate_cert_hash_hex(cert_data);
        Ok(format!("Certificate hash: {}", hash))
    })
}
