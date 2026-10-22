// Module: common\buf\copy_test.rs
// 1:1 Rust unit test suite corresponding to Go common\buf\copy_test.go

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicI64, Ordering};
    use std::time::Duration;
    use tokio::io::AsyncWriteExt;

    use super::super::buffer::Buffer;
    use super::super::copy::{CopyOptions, copy, copy_once_timeout, copy_stream};
    use super::super::multi_buffer::MultiBuffer;
    use super::super::reader::SingleReader;
    use super::super::writer::SequentialWriter;

    #[tokio::test]
    async fn test_copy_stream_duplex() {
        let (mut client_r, mut client_w) = tokio::io::duplex(64);
        let (mut server_r, mut server_w) = tokio::io::duplex(64);

        tokio::spawn(async move {
            client_w.write_all(b"ping").await.unwrap();
            client_w.flush().await.unwrap();
        });

        tokio::spawn(async move {
            copy_stream(&mut client_r, &mut server_w, 2048, CopyOptions::default())
                .await
                .unwrap();
        });

        let mut buf = [0u8; 4];
        tokio::io::AsyncReadExt::read_exact(&mut server_r, &mut buf)
            .await
            .unwrap();
        assert_eq!(&buf, b"ping");
    }

    #[tokio::test]
    async fn test_copy_multibuffer_reader_writer() {
        let mut source = Cursor::new(b"copy test payload".to_vec());
        let mut reader = SingleReader::new(&mut source);

        let sink = Cursor::new(Vec::new());
        let mut writer = SequentialWriter::new(sink);

        let size_counter = Arc::new(AtomicI64::new(0));
        let options = CopyOptions::new().with_size_counter(size_counter.clone());

        let copied = copy(&mut reader, &mut writer, &options).await.unwrap();
        assert_eq!(copied, 17);
        assert_eq!(size_counter.load(Ordering::Relaxed), 17);
    }

    #[tokio::test]
    async fn test_copy_once_timeout_reader() {
        let (stream_a, stream_b) = tokio::io::duplex(1024);
        let mut conn_a = crate::common::singbridge::Conn::new(Box::pin(stream_a));
        let mut conn_b = crate::common::singbridge::Conn::new(Box::pin(stream_b));

        let mut mb = MultiBuffer::new();
        mb.push(Buffer::from_bytes(b"timeout copy"));

        use crate::common::buf::io::Writer;
        conn_a.write_multi_buffer(mb).await.unwrap();

        let sink = Cursor::new(Vec::new());
        let mut writer = SequentialWriter::new(sink);

        let len = copy_once_timeout(&mut conn_b, &mut writer, Duration::from_secs(2))
            .await
            .unwrap();
        assert_eq!(len, 12);
    }
}
