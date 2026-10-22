#[cfg(test)]
mod tests {
    use crate::app::stats::Counter;
    use crate::common::buf::{
        BufferedReader, BufferedWriter, CopyOptions, VectorReader, copy_stream,
    };
    use crate::common::signal::ActivityTimer;
    use std::sync::Arc;
    use std::sync::atomic::AtomicI64;
    use std::time::Duration;
    use tokio::io::AsyncReadExt;

    #[tokio::test]
    async fn test_copy_stream_with_timer_and_counter() {
        let payload = b"Hello, Xray Rust 1:1 stream copy engine!";
        let (mut client, server) = tokio::io::duplex(128);
        let (server_r, client_w) = tokio::io::duplex(128);

        let timer = Arc::new(ActivityTimer::new(Duration::from_secs(60)));
        let counter = Arc::new(Counter::new());
        let size_counter = Arc::new(AtomicI64::new(0));

        let options = CopyOptions {
            timer: Some(timer.clone()),
            counter: Some(counter.clone()),
            size_counter: Some(size_counter.clone()),
        };

        tokio::spawn(async move {
            tokio::io::AsyncWriteExt::write_all(&mut client, payload)
                .await
                .unwrap();
        });

        let copied = copy_stream(server, client_w, 64, options).await.unwrap();
        assert_eq!(copied, payload.len() as u64);
        assert_eq!(counter.value(), payload.len() as i64);
        assert_eq!(
            size_counter.load(std::sync::atomic::Ordering::Relaxed),
            payload.len() as i64
        );

        let mut read_buf = vec![0u8; payload.len()];
        let mut server_r_pinned = server_r;
        server_r_pinned.read_exact(&mut read_buf).await.unwrap();
        assert_eq!(&read_buf, payload);
    }

    #[tokio::test]
    async fn test_buffered_reader_and_writer() {
        let (mut client_w, server_r) = tokio::io::duplex(64);
        let (server_w, mut client_r) = tokio::io::duplex(64);

        let mut reader = BufferedReader::new(server_r, 32);
        let mut writer = BufferedWriter::new(server_w);

        tokio::spawn(async move {
            tokio::io::AsyncWriteExt::write_all(&mut client_w, b"buffer-test-payload")
                .await
                .unwrap();
        });

        let buf = reader.read_buffer().await.unwrap().unwrap();
        assert_eq!(buf.as_slice(), b"buffer-test-payload");

        writer.write_buffer(&buf).await.unwrap();
        writer.flush().await.unwrap();

        let mut out = [0u8; 19];
        client_r.read_exact(&mut out).await.unwrap();
        assert_eq!(&out, b"buffer-test-payload");
    }

    #[tokio::test]
    async fn test_vector_reader_multi_buffer() {
        let (mut client_w, server_r) = tokio::io::duplex(64);
        let mut vec_reader = VectorReader::new(server_r, 16);

        tokio::spawn(async move {
            tokio::io::AsyncWriteExt::write_all(&mut client_w, b"vector-chunking-test")
                .await
                .unwrap();
        });

        let mb = vec_reader.read_multi_buffer(4).await.unwrap().unwrap();
        assert_eq!(mb.len(), 20);
        let flat = mb.to_vec();
        assert_eq!(&flat, b"vector-chunking-test");
    }
}
