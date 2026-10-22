#[cfg(test)]
mod tests {
    use crate::transport::internet::finalmask::SalamanderObfuscator;
    use crate::transport::internet::happy_eyeballs::sort_ips;
    use crate::transport::internet::tls::pin::generate_cert_hash_hex;
    use std::net::IpAddr;

    #[test]
    fn test_happy_eyeballs_ip_sorting_and_interleave() {
        let ip_v4_1: IpAddr = "1.1.1.1".parse().unwrap();
        let ip_v4_2: IpAddr = "1.0.0.1".parse().unwrap();
        let ip_v6_1: IpAddr = "2606:4700:4700::1111".parse().unwrap();
        let ip_v6_2: IpAddr = "2606:4700:4700::1001".parse().unwrap();

        let all_ips = vec![ip_v4_1, ip_v4_2, ip_v6_1, ip_v6_2];

        // IPv4 first interleaved
        let interleaved_v4 = sort_ips(&all_ips, false, true);
        assert_eq!(interleaved_v4[0], ip_v4_1);
        assert_eq!(interleaved_v4[1], ip_v6_1);
        assert_eq!(interleaved_v4[2], ip_v4_2);
        assert_eq!(interleaved_v4[3], ip_v6_2);

        // IPv6 priority interleaved
        let interleaved_v6 = sort_ips(&all_ips, true, true);
        assert_eq!(interleaved_v6[0], ip_v6_1);
        assert_eq!(interleaved_v6[1], ip_v4_1);
    }

    #[test]
    fn test_tls_cert_fingerprint_hashing() {
        let dummy_der = b"dummy-x509-certificate-asn1-der-payload";
        let hex_fingerprint = generate_cert_hash_hex(dummy_der);
        assert_eq!(hex_fingerprint.len(), 64); // SHA256 hex length
    }

    #[test]
    fn test_salamander_obfuscate_deobfuscate_roundtrip() {
        let psk = b"super-secret-salamander-psk".to_vec();
        let obfs = SalamanderObfuscator::new(psk).unwrap();

        let original = b"Sensitive payload needing Blake2b XOR obfuscation";
        let obfuscated = obfs.obfuscate(original);

        assert_ne!(&obfuscated[8..], original);
        assert_eq!(obfuscated.len(), original.len() + 8);

        let recovered = obfs.deobfuscate(&obfuscated).unwrap();
        assert_eq!(&recovered, original);
    }
}
