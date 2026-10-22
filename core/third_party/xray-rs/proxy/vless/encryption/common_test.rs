// Module: proxy\vless\encryption\common_test.rs
// Comprehensive unit tests for VLESS Common encryption, AEAD seal/open, and CommonConn streaming

#[cfg(test)]
mod tests {
    use super::super::common::{
        CommonConn, VlessAead, create_padding, decode_header, encode_header, increase_nonce,
        parse_padding,
    };
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[test]
    fn test_header_encode_decode() {
        let mut header = [0u8; 5];
        encode_header(&mut header, 1024);
        assert_eq!(&header[..3], &[23, 3, 3]);

        let len = decode_header(&header).expect("Valid header decode");
        assert_eq!(len, 1024);

        // Invalid record type
        let invalid_header = [22, 3, 3, 0, 100];
        assert!(decode_header(&invalid_header).is_err());

        // Length too small
        let too_small = [23, 3, 3, 0, 10];
        assert!(decode_header(&too_small).is_err());
    }

    #[test]
    fn test_increase_nonce() {
        let mut nonce = [0u8; 12];
        increase_nonce(&mut nonce);
        assert_eq!(nonce[11], 1);

        nonce[11] = 0xFF;
        increase_nonce(&mut nonce);
        assert_eq!(nonce[11], 0);
        assert_eq!(nonce[10], 1);
    }

    #[test]
    fn test_aead_seal_and_open() {
        let key = [42u8; 32];
        let mut aead_sender = VlessAead::new(&key, true); // AES
        let mut aead_receiver = VlessAead::new(&key, true);

        let msg = b"Secret VLESS MLKEM Payload";
        let aad = b"header-aad";

        let ciphertext = aead_sender.seal(msg, aad).unwrap();
        let decrypted = aead_receiver.open(&ciphertext, aad).unwrap();
        assert_eq!(&decrypted, msg);
    }

    #[test]
    fn test_parse_and_create_padding() {
        let padding_str = "100-50-200.75-10-30";
        let (lens, gaps) = parse_padding(padding_str).unwrap();
        assert_eq!(lens.len(), 1);
        assert_eq!(gaps.len(), 1);

        let (total_len, out_lens, out_gaps) = create_padding(&lens, &gaps);
        assert!(total_len >= 50);
        assert_eq!(out_lens.len(), 1);
        assert_eq!(out_gaps.len(), 1);
    }

    #[tokio::test]
    async fn test_common_conn_duplex_stream() {
        let (raw_client, raw_server) = tokio::io::duplex(4096);
        let key = [77u8; 32];

        let mut client_conn = CommonConn::new(raw_client, &key, false); // ChaCha
        let mut server_conn = CommonConn::new(raw_server, &key, false);

        let payload = b"Hello, encrypted VLESS CommonConn stream!";
        client_conn.write_all(payload).await.unwrap();
        client_conn.flush().await.unwrap();

        let mut recv = vec![0u8; payload.len()];
        server_conn.read_exact(&mut recv).await.unwrap();
        assert_eq!(&recv, payload);
    }
}
