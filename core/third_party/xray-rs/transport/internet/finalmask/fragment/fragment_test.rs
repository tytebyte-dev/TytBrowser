// Module: transport\internet\finalmask\fragment\fragment_test.rs
#[cfg(test)]
mod tests {
    use super::super::config_pb::Config;
    use super::super::conn::FragmentConn;
    use crate::transport::internet::finalmask::finalmask::TcpMaskConn;
    use tokio::io::{AsyncReadExt, duplex};

    #[test]
    fn test_fragment_config_defaults() {
        let config = Config::default();
        assert_eq!(config.packets_from, 0);
        assert_eq!(config.packets_to, 0);
        assert_eq!(config.length_min, 0);
        assert_eq!(config.length_max, 0);
        assert_eq!(config.delay_min, 0);
        assert_eq!(config.delay_max, 0);
        assert_eq!(config.max_split_min, 0);
        assert_eq!(config.max_split_max, 0);
    }

    #[tokio::test]
    async fn test_fragment_conn_client_server_wrap() {
        let (client_io, server_io) = duplex(1024);
        let config = Config {
            packets_from: 0,
            packets_to: 1,
            length_min: 10,
            length_max: 20,
            delay_min: 0,
            delay_max: 0,
            max_split_min: 3,
            max_split_max: 5,
        };

        let client_conn = config.wrap_conn_client(Box::pin(client_io)).unwrap();
        let server_conn = config.wrap_conn_server(Box::pin(server_io)).unwrap();

        assert!(!client_conn.server);
        assert!(server_conn.server);
        assert!(TcpMaskConn::splice(&client_conn));
        assert!(!TcpMaskConn::splice(&server_conn));
    }

    #[tokio::test]
    async fn test_fragment_conn_tls_client_hello() {
        let (client_io, mut server_io) = duplex(4096);
        let config = Config {
            packets_from: 0,
            packets_to: 1,
            length_min: 5,
            length_max: 10,
            delay_min: 0,
            delay_max: 0,
            max_split_min: 2,
            max_split_max: 4,
        };

        let mut client_conn = FragmentConn::new_client(config, Box::pin(client_io));

        // Construct mock TLS ClientHello record:
        // Byte 0: 22 (Handshake)
        // Byte 1-2: 0x03, 0x01 (TLS 1.0)
        // Byte 3-4: length of handshake payload (e.g. 20 bytes)
        // Byte 5..25: Handshake payload
        let mut tls_record = vec![22, 0x03, 0x01, 0x00, 20];
        let payload = b"0123456789abcdefghij";
        tls_record.extend_from_slice(payload);

        let write_task = tokio::spawn(async move {
            client_conn.write_fragmented(&tls_record).await.unwrap();
        });

        // Server reads the fragmented TLS records and reassembles payload
        let read_task = tokio::spawn(async move {
            let mut received_payload = Vec::new();
            let mut buf = vec![0u8; 1024];
            let mut total_read = 0;
            // The fragmented records each have a 5-byte header, so total bytes > 25
            while received_payload.len() < payload.len() {
                let n = server_io.read(&mut buf).await.unwrap();
                assert!(n > 0);
                total_read += n;
                // Parse TLS record chunks: each starts with 22, 0x03, 0x01, len_hi, len_lo
                let mut offset = 0;
                while offset + 5 <= n {
                    assert_eq!(buf[offset], 22);
                    assert_eq!(buf[offset + 1], 0x03);
                    assert_eq!(buf[offset + 2], 0x01);
                    let chunk_len = ((buf[offset + 3] as usize) << 8) | (buf[offset + 4] as usize);
                    let chunk_end = offset + 5 + chunk_len;
                    assert!(chunk_end <= n);
                    received_payload.extend_from_slice(&buf[offset + 5..chunk_end]);
                    offset = chunk_end;
                }
            }
            assert_eq!(received_payload, payload);
            assert!(total_read > 25); // Verified that it was indeed fragmented into multiple records
        });

        write_task.await.unwrap();
        read_task.await.unwrap();
    }

    #[tokio::test]
    async fn test_fragment_conn_generic_packets() {
        let (client_io, mut server_io) = duplex(4096);
        let config = Config {
            packets_from: 1,
            packets_to: 2,
            length_min: 8,
            length_max: 12,
            delay_min: 0,
            delay_max: 0,
            max_split_min: 2,
            max_split_max: 3,
        };

        let mut client_conn = FragmentConn::new_client(config, Box::pin(client_io));
        let data = b"Hello, this is a generic packet stream destined to be fragmented into chunks!";

        let data_clone = data.to_vec();
        let write_task = tokio::spawn(async move {
            client_conn.write_fragmented(&data_clone).await.unwrap();
        });

        let read_task = tokio::spawn(async move {
            let mut received = Vec::new();
            let mut buf = vec![0u8; 256];
            while received.len() < data.len() {
                let n = server_io.read(&mut buf).await.unwrap();
                if n == 0 {
                    break;
                }
                received.extend_from_slice(&buf[..n]);
            }
            assert_eq!(received, data);
        });

        write_task.await.unwrap();
        read_task.await.unwrap();
    }
}
