#[cfg(test)]
mod tests {
    use crate::common::buf::Buffer;
    use crate::common::crypto::AesGcmCipher;
    use crate::common::net::{Address, Destination, Network};
    use crate::common::protocol::{BittorrentSniffer, Timestamp, UdpPacket};
    use std::net::Ipv4Addr;

    #[test]
    fn test_aes_gcm_128_and_256_roundtrip() {
        let key16 = b"0123456789abcdef";
        let nonce = b"123456789012";
        let plaintext = b"Hello, AES-128-GCM authenticated cipher!";

        let encrypted = AesGcmCipher::encrypt_128(key16, nonce, plaintext).unwrap();
        let decrypted = AesGcmCipher::decrypt_128(key16, nonce, &encrypted).unwrap();
        assert_eq!(&decrypted, plaintext);

        let key32 = b"0123456789abcdef0123456789abcdef";
        let enc256 = AesGcmCipher::encrypt_256(key32, nonce, plaintext).unwrap();
        let dec256 = AesGcmCipher::decrypt_256(key32, nonce, &enc256).unwrap();
        assert_eq!(&dec256, plaintext);
    }

    #[test]
    fn test_bittorrent_and_utp_sniffing() {
        let mut bt_header = vec![19u8];
        bt_header.extend_from_slice(b"BitTorrent protocol");
        bt_header.extend_from_slice(&[0u8; 8]);

        assert_eq!(BittorrentSniffer::sniff(&bt_header).unwrap(), "bittorrent");

        let utp_header = vec![
            0x11, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        assert_eq!(BittorrentSniffer::sniff(&utp_header).unwrap(), "utp");

        let invalid = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
        assert!(BittorrentSniffer::sniff(invalid).is_err());
    }

    #[test]
    fn test_protocol_timestamp_and_jitter() {
        let now = Timestamp::now();
        assert!(now.is_valid(120));

        let jittered = now.with_jitter(10);
        assert!((jittered.0 - now.0).abs() <= 10);
    }

    #[test]
    fn test_udp_packet_creation() {
        let buf = Buffer::from_bytes(b"dns-query-packet");
        let src = Destination::new(Address::Ipv4(Ipv4Addr::new(127, 0, 0, 1)), 54321);
        let dst = Destination::udp(Address::Ipv4(Ipv4Addr::new(8, 8, 8, 8)), 53);

        let packet = UdpPacket::new(buf, src, dst);
        assert_eq!(packet.target.network, Network::Udp);
        assert_eq!(packet.target.port, 53);
        assert_eq!(packet.payload.as_slice(), b"dns-query-packet");
    }
}
