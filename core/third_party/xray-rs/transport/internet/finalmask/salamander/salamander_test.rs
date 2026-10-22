// Module: transport\internet\finalmask\salamander\salamander_test.rs

#[cfg(test)]
mod tests {
    use super::super::config::SalamanderConfig;
    use super::super::conn::SalamanderPacketConn;
    use super::super::salamander::{SM_SALT_LEN, SalamanderObfuscator};

    #[test]
    fn test_salamander_obfuscator_roundtrip() {
        let obf = SalamanderObfuscator::new(b"average_password".to_vec()).unwrap();
        let payload = b"Hello Salamander 1:1 XOR obfuscation with BLAKE2b!";

        let mut out = vec![0u8; payload.len() + SM_SALT_LEN];
        let n_enc = obf.obfuscate_slice(payload, &mut out);
        assert_eq!(n_enc, payload.len() + SM_SALT_LEN);

        let mut dec = vec![0u8; payload.len()];
        let n_dec = obf.deobfuscate_slice(&out[..n_enc], &mut dec);
        assert_eq!(n_dec, payload.len());
        assert_eq!(&dec[..n_dec], payload);

        // Also test convenience methods
        let enc_vec = obf.obfuscate(payload);
        assert_eq!(enc_vec.len(), payload.len() + SM_SALT_LEN);
        let dec_vec = obf.deobfuscate(&enc_vec).unwrap();
        assert_eq!(&dec_vec, payload);
    }

    #[test]
    fn test_salamander_bounce_short_packet() {
        let obf = SalamanderObfuscator::new(b"average_password".to_vec()).unwrap();
        let short_buf = vec![0u8; 8];
        let mut out = vec![0u8; 8];
        let n = obf.deobfuscate_slice(&short_buf, &mut out);
        assert_eq!(n, 0);

        assert!(obf.deobfuscate(&short_buf).is_err());
    }

    #[test]
    fn test_salamander_conn_packet_wrapper() {
        let config = SalamanderConfig::new("average_password");
        let conn = SalamanderPacketConn::new_client(&config).unwrap();
        let payload = b"PacketConn layer test payload for UDP tunneling";

        let masked = conn.mask(payload);
        assert_eq!(masked.len(), payload.len() + conn.size());

        let unmasked = conn.unmask(&masked).unwrap();
        assert_eq!(unmasked, payload);
    }
}
