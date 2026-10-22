// Module: transport\internet\tls\config_test.rs
// 1:1 Rust unit test suite corresponding to Go transport\internet\tls\config_test.go

#[cfg(test)]
mod tests {
    use super::super::config::{Certificate, CertificateUsage, TlsConfig};

    #[test]
    fn test_tls_config_default() {
        let cfg = TlsConfig::default();
        assert!(cfg.server_name.is_empty());
        assert!(!cfg.allow_insecure);
        assert!(cfg.certificate.is_empty());
        assert!(cfg.next_protocol.is_empty());
    }

    #[test]
    fn test_certificate_filtering_by_usage() {
        let enc_cert =
            Certificate::new(vec![1, 2, 3], vec![4, 5, 6], CertificateUsage::Encipherment);
        let verify_cert =
            Certificate::new(vec![7, 8], vec![9, 10], CertificateUsage::AuthorityVerify);
        let issue_cert = Certificate::new(vec![11], vec![12], CertificateUsage::AuthorityIssue);

        let cfg = TlsConfig {
            certificate: vec![enc_cert.clone(), verify_cert.clone(), issue_cert.clone()],
            ..Default::default()
        };

        let server_certs = cfg.build_certificates();
        assert_eq!(server_certs.len(), 1);
        assert_eq!(server_certs[0].usage, CertificateUsage::Encipherment);

        let client_roots = cfg.get_client_verify_certs();
        assert_eq!(client_roots.len(), 1);
        assert_eq!(client_roots[0].usage, CertificateUsage::AuthorityVerify);
    }

    #[test]
    fn test_certificate_issuing_for_sni() {
        let ca_cert = Certificate::new(
            b"dummy ca".to_vec(),
            b"dummy ca key".to_vec(),
            CertificateUsage::AuthorityIssue,
        );
        let mut cfg = TlsConfig {
            certificate: vec![ca_cert],
            ..Default::default()
        };

        let issued = cfg
            .get_certificate_for_sni("www.example.com")
            .expect("should issue certificate");
        assert_eq!(issued.usage, CertificateUsage::Encipherment);
        assert!(!issued.certificate.is_empty());
        assert!(!issued.key.is_empty());

        // Check cached cert
        let cached = cfg
            .get_certificate_for_sni("www.example.com")
            .expect("should retrieve cached certificate");
        assert_eq!(issued.certificate, cached.certificate);
    }
}
