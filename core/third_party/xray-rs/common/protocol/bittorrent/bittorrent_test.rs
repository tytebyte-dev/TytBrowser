// Module: common\protocol\bittorrent\bittorrent_test.rs
// 1:1 Rust unit test suite corresponding to Go common\protocol\bittorrent\bittorrent_test.go

#[cfg(test)]
mod tests {
    use super::super::bittorrent::BittorrentSniffer;

    #[test]
    fn test_bittorrent_sniff_valid() {
        let mut packet = vec![19u8];
        packet.extend_from_slice(b"BitTorrent protocol");
        packet.extend_from_slice(&[0u8; 8]); // 8 reserved bytes
        packet.extend_from_slice(&[0xabu8; 20]); // info_hash
        packet.extend_from_slice(&[0xcdu8; 20]); // peer_id

        let res = BittorrentSniffer::sniff(&packet);
        assert_eq!(res.unwrap(), "bittorrent");
    }

    #[test]
    fn test_utp_sniff_valid() {
        // ver = 1, type = 0 (ST_DATA) -> 0x01
        // ext = 0
        let packet = vec![
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let res = BittorrentSniffer::sniff(&packet);
        assert_eq!(res.unwrap(), "utp");
    }

    #[test]
    fn test_bittorrent_sniff_invalid() {
        let packet = b"GET /index.html HTTP/1.1\r\n\r\n";
        assert!(BittorrentSniffer::sniff(packet).is_err());
    }
}
