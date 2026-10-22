// Module: transport\internet\finalmask\tcp_test.rs
#[cfg(test)]
mod tests {
    use super::super::finalmask::{
        FINALMASK_VERSION, TcpMaskConn, TcpmaskManager, unwrap_tcp_mask,
    };
    use super::super::header::custom::config::{TCPConfig, TCPItem, TCPSequence};
    use super::super::header::custom::tcp::{
        TcpCustomConn, client_handshake, read_sequence, server_handshake, write_sequence,
    };
    use tokio::io::{AsyncReadExt, AsyncWriteExt, duplex};

    #[test]
    fn test_finalmask_version() {
        assert_eq!(FINALMASK_VERSION, 1);
    }

    #[tokio::test]
    async fn test_tcp_custom_sequence_roundtrip() {
        let (mut client, mut server) = duplex(1024);

        let seq = TCPSequence {
            sequence: vec![
                TCPItem {
                    delay_min: 0,
                    delay_max: 0,
                    rand: 0,
                    rand_min: 0,
                    rand_max: 0,
                    packet: b"HELLO-SERVER".to_vec(),
                },
                TCPItem {
                    delay_min: 0,
                    delay_max: 0,
                    rand: 4,
                    rand_min: 0x10,
                    rand_max: 0x20,
                    packet: vec![],
                },
            ],
        };

        let seq_clone = TCPSequence {
            sequence: vec![
                TCPItem {
                    delay_min: 0,
                    delay_max: 0,
                    rand: 0,
                    rand_min: 0,
                    rand_max: 0,
                    packet: b"HELLO-SERVER".to_vec(),
                },
                TCPItem {
                    delay_min: 0,
                    delay_max: 0,
                    rand: 4,
                    rand_min: 0,
                    rand_max: 0,
                    packet: vec![],
                },
            ],
        };

        let writer_task = tokio::spawn(async move {
            write_sequence(&mut client, &seq).await.unwrap();
        });

        let reader_task = tokio::spawn(async move {
            read_sequence(&mut server, &seq_clone).await.unwrap();
        });

        writer_task.await.unwrap();
        reader_task.await.unwrap();
    }

    #[tokio::test]
    async fn test_tcp_custom_full_handshake() {
        let (mut client, mut server) = duplex(2048);

        let config = TCPConfig {
            clients: vec![TCPSequence {
                sequence: vec![TCPItem {
                    delay_min: 0,
                    delay_max: 0,
                    rand: 0,
                    rand_min: 0,
                    rand_max: 0,
                    packet: b"REQ-AUTH-TOKEN".to_vec(),
                }],
            }],
            servers: vec![TCPSequence {
                sequence: vec![TCPItem {
                    delay_min: 0,
                    delay_max: 0,
                    rand: 0,
                    rand_min: 0,
                    rand_max: 0,
                    packet: b"RESP-AUTH-OK".to_vec(),
                }],
            }],
            errors: vec![],
        };

        let client_cfg = config.clone();
        let server_cfg = config.clone();

        let client_task = tokio::spawn(async move {
            client_handshake(&mut client, &client_cfg).await.unwrap();
            let mut conn = TcpCustomConn::new(client);
            conn.write_all(b"payload-data").await.unwrap();
            conn.flush().await.unwrap();
        });

        let server_task = tokio::spawn(async move {
            server_handshake(&mut server, &server_cfg).await.unwrap();
            let mut conn = TcpCustomConn::new(server);
            let mut buf = vec![0u8; 12];
            conn.read_exact(&mut buf).await.unwrap();
            assert_eq!(&buf, b"payload-data");
        });

        client_task.await.unwrap();
        server_task.await.unwrap();
    }

    struct MockConn;
    impl TcpMaskConn for MockConn {}

    #[test]
    fn test_tcp_mask_manager_and_unwrap() {
        let mgr = TcpmaskManager::new(vec![]);
        assert!(mgr.is_empty());
        assert_eq!(mgr.len(), 0);

        let conn = MockConn;
        assert!(unwrap_tcp_mask(&conn));
    }
}
