// Module: proxy\vless\flow\vision_test.rs
// Unit tests for VisionStream, padding framing, unpadding, and Direct Copy transition

#[cfg(test)]
mod tests {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use uuid::Uuid;

    use super::super::vision::{VisionContext, VisionFilter, VisionStream};

    #[tokio::test]
    async fn test_vision_padded_framing_and_unpadding() {
        let (client_raw, server_raw) = tokio::io::duplex(4096);

        let user_uuid = Uuid::new_v4().as_bytes().to_vec();

        let client_ctx = VisionContext::new(user_uuid.clone(), true);
        let server_ctx = VisionContext::new(user_uuid.clone(), false);

        let mut client_stream = VisionStream::new(client_raw, client_ctx, false);
        let mut server_stream = VisionStream::new(server_raw, server_ctx, true);

        // Send payload through client
        let msg = b"Testing XTLS-Vision Flow Data Transmission!";
        client_stream
            .write_all(msg)
            .await
            .expect("Client write should succeed");
        client_stream
            .flush()
            .await
            .expect("Client flush should succeed");

        // Read through server stream - should automatically unpad
        let mut received = vec![0u8; msg.len()];
        server_stream
            .read_exact(&mut received)
            .await
            .expect("Server read should succeed");

        assert_eq!(&received, msg);
    }

    #[test]
    fn test_vision_filter_inspect_tls_record_length() {
        let mut record = vec![0x16, 0x03, 0x03];
        record.extend_from_slice(&(42u16).to_be_bytes()); // length 42
        record.extend_from_slice(&[0u8; 42]);

        let len = VisionFilter::inspect_tls_record_length(&record);
        assert_eq!(len, Some(47)); // 5 + 42

        assert_eq!(VisionFilter::inspect_tls_record_length(&[0x16, 0x03]), None);
    }

    #[tokio::test]
    async fn test_vision_direct_copy_mode() {
        let (client_raw, server_raw) = tokio::io::duplex(4096);
        let user_uuid = Uuid::new_v4().as_bytes().to_vec();

        let client_ctx = VisionContext::new(user_uuid.clone(), true);
        let server_ctx = VisionContext::new(user_uuid.clone(), false);

        // Set direct copy explicitly to simulate post-handshake mode
        client_ctx.set_direct_copy(true);
        server_ctx.set_direct_copy(true);

        assert!(client_ctx.is_direct_copy());
        assert!(server_ctx.is_direct_copy());

        let mut client_stream = VisionStream::new(client_raw, client_ctx, false);
        let mut server_stream = VisionStream::new(server_raw, server_ctx, true);

        let raw_data = b"Pure direct payload without any padding framing";
        client_stream.write_all(raw_data).await.unwrap();

        let mut recv = vec![0u8; raw_data.len()];
        server_stream.read_exact(&mut recv).await.unwrap();

        assert_eq!(&recv, raw_data);
    }
}
