// Module: transport\internet\finalmask\sudoku\sudoku_test.rs

#[cfg(test)]
mod tests {
    use super::super::config::SudokuConfig;
    use super::super::conn_tcp::SudokuTcpConn;
    use super::super::conn_tcp_packed::{PackedEncoder, PackedStreamDecoder};
    use super::super::conn_udp::SudokuUdpConn;
    use super::super::table::get_tables;

    #[test]
    fn test_sudoku_table_generation() {
        let config = SudokuConfig::new("sudoku-test-password-123");
        let tables = get_tables(&config).expect("failed to get tables");
        assert_eq!(tables.len(), 1);

        let table = &tables[0];
        assert_eq!(table.encode.len(), 256);
        for b in 0..256 {
            assert!(
                !table.encode[b].is_empty(),
                "byte {} has empty encode clues",
                b
            );
        }
        assert!(table.decode.len() >= 256);
    }

    #[test]
    fn test_sudoku_ascii_roundtrip() {
        let mut config = SudokuConfig::new("sudoku-ascii-secret");
        config.ascii = "prefer_ascii".into();
        config.padding_min = 10;
        config.padding_max = 20;

        let udp = SudokuUdpConn::new(&config).expect("failed to init udp");
        let payload = b"Hello, World! This is an ASCII-favored Sudoku masking test in Rust.";
        let masked = udp.mask(payload).expect("mask failed");

        let ascii_chars = masked
            .iter()
            .filter(|&&b| b >= 0x20 && b <= 0x7e || b == b'\n')
            .count();
        let ratio = (ascii_chars as f64) / (masked.len() as f64);
        assert!(ratio >= 0.95, "ASCII ratio {} too low", ratio);

        let unmasked = udp.unmask(&masked).expect("unmask failed");
        assert_eq!(&unmasked[..], payload);
    }

    #[test]
    fn test_sudoku_entropy_roundtrip() {
        let mut config = SudokuConfig::new("sudoku-entropy-secret");
        config.ascii = "prefer_entropy".into();
        config.padding_min = 15;
        config.padding_max = 30;

        let udp = SudokuUdpConn::new(&config).expect("failed to init udp");
        let payload = vec![
            0xca, 0xfe, 0xba, 0xbe, 0x01, 0x02, 0x03, 0xff, 0x00, 0x55, 0xaa,
        ];
        let masked = udp.mask(&payload).expect("mask failed");
        let unmasked = udp.unmask(&masked).expect("unmask failed");
        assert_eq!(unmasked, payload);
    }

    #[test]
    fn test_sudoku_custom_tables_rotation() {
        let mut config = SudokuConfig::new("sudoku-custom-tables-secret");
        config.ascii = "prefer_entropy".into();
        config.custom_tables = vec![
            "xpxvvpvv".into(),
            "vxpvxvvp".into(),
            "pxvvxvvp".into(),
            "vpxvxvpv".into(),
            "xvpvvxpv".into(),
            "vvxpxpvv".into(),
        ];

        let tables = get_tables(&config).expect("failed to load custom tables");
        assert_eq!(tables.len(), 6);

        let udp = SudokuUdpConn::new(&config).expect("failed to init udp");
        let payload = b"Testing custom sudoku tables rotation pattern across 6 unique tables!";
        let masked = udp.mask(payload).expect("mask failed");
        let unmasked = udp.unmask(&masked).expect("unmask failed");
        assert_eq!(&unmasked[..], payload);
    }

    #[test]
    fn test_sudoku_packed_tcp_roundtrip() {
        let config = SudokuConfig::new("sudoku-packed-secret");
        let tables = get_tables(&config).expect("tables");

        let mut encoder = PackedEncoder::new(&tables, 0, 0);
        let mut decoder = PackedStreamDecoder::new(&tables);

        let payload = b"High-throughput packed downlink payload data with arbitrary binary \x00\xff\x88\x77\x11";
        let packed = encoder.encode(payload).expect("encode");

        let mut decoded = Vec::new();
        decoder.decode_chunk(&packed, &mut decoded).expect("decode");
        assert_eq!(&decoded[..], payload);
    }

    #[test]
    fn test_sudoku_directional_tcp_conn() {
        let mut config = SudokuConfig::new("sudoku-directional-secret");
        config.padding_min = 5;
        config.padding_max = 10;

        let mut client = SudokuTcpConn::new_client(&config).expect("client");
        let mut server = SudokuTcpConn::new_server(&config).expect("server");

        // Client to Server (Uplink: client writes pure, server reads pure)
        let uplink_data = b"Uplink request from client to server";
        let client_sent = client.encode_stream(uplink_data).expect("client send");
        let mut server_buf = vec![0u8; 1024];
        let n1 = server
            .feed_and_read(&client_sent, &mut server_buf)
            .expect("server read");
        assert_eq!(&server_buf[..n1], uplink_data);

        // Server to Client (Downlink: server writes packed, client reads packed)
        let downlink_data = b"Downlink response from server to client with high compression";
        let server_sent = server.encode_stream(downlink_data).expect("server send");
        let mut client_buf = vec![0u8; 1024];
        let n2 = client
            .feed_and_read(&server_sent, &mut client_buf)
            .expect("client read");
        assert_eq!(&client_buf[..n2], downlink_data);
    }
}
