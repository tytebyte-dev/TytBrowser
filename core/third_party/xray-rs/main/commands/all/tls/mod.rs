pub mod cert;
pub mod ech;
pub mod hash;
pub mod ping;
pub mod tls;

pub use tls::cmd_tls;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tls_commands() {
        let cmd = cmd_tls();
        assert_eq!(cmd.name, "tls");
        assert_eq!(cmd.subcommands.len(), 4);

        let cert_res = cmd.execute(&["cert", "test.local"]).unwrap();
        assert!(cert_res.contains("BEGIN CERTIFICATE"));
        assert!(cert_res.contains("BEGIN PRIVATE KEY"));

        let ech_res = cmd.execute(&["ech"]).unwrap();
        assert!(ech_res.contains("ECH Private Key:"));

        let hash_res = cmd.execute(&["hash", "dummy-cert-data"]).unwrap();
        assert!(hash_res.contains("Certificate hash:"));

        let ping_res = cmd.execute(&["ping", "127.0.0.1:443"]).unwrap();
        assert!(ping_res.contains("TLS ping to 127.0.0.1:443"));
    }
}
