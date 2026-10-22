// Module: common\protocol\tls\cert\cert_test.rs
// 1:1 Rust unit test suite corresponding to Go common\protocol\tls\cert\cert_test.go

#[cfg(test)]
mod tests {
    use super::super::cert::{Certificate, parse_certificate};

    #[test]
    fn test_certificate_fingerprint_and_pem_roundtrip() {
        let cert = Certificate::new(vec![1, 2, 3, 4, 5, 6], vec![7, 8, 9, 10]);
        let fp = cert.sha256_fingerprint();
        assert_eq!(fp.len(), 32);

        let (cpem, kpem) = cert.to_pem();
        assert!(cpem.starts_with("-----BEGIN CERTIFICATE-----"));
        assert!(kpem.starts_with("-----BEGIN PRIVATE KEY-----"));

        let parsed = parse_certificate(&cpem, &kpem).expect("parse certificate");
        assert_eq!(parsed.certificate, cert.certificate);
        assert_eq!(parsed.private_key, cert.private_key);
        assert_eq!(parsed.sha256_fingerprint(), fp);
    }
}
