// Module: testing/scenarios/pipe_full_test.rs
// 1:1 unit test suite corresponding to Go transport/pipe/pipe_test.go

#[cfg(test)]
mod tests {
    use crate::common::buf::{Buffer, MultiBuffer};
    use crate::common::errors::Error;
    use crate::transport::pipe::{
        PipeOption, discard_overflow, new_pipe, new_with_options, with_size_limit,
    };

    #[tokio::test]
    async fn test_pipe_read_write() {
        let (reader, writer) = new_with_options(vec![with_size_limit(1024)]);

        let mut b1 = MultiBuffer::new();
        b1.push(Buffer::from_bytes(b"abcd"));
        writer.write_multi_buffer(b1).await.unwrap();

        let mut b2 = MultiBuffer::new();
        b2.push(Buffer::from_bytes(b"efg"));
        writer.write_multi_buffer(b2).await.unwrap();

        let rb = reader.read_multi_buffer().await.unwrap();
        assert_eq!(rb.to_vec(), b"abcdefg");
    }

    #[tokio::test]
    async fn test_pipe_interrupt() {
        let (reader, writer) = new_with_options(vec![with_size_limit(1024)]);

        let mut b = MultiBuffer::new();
        b.push(Buffer::from_bytes(b"abcd"));
        writer.write_multi_buffer(b).await.unwrap();

        writer.interrupt().await;

        let err = reader.read_multi_buffer().await.unwrap_err();
        assert!(matches!(err, Error::Closed));
    }

    #[tokio::test]
    async fn test_pipe_close() {
        let (reader, writer) = new_with_options(vec![with_size_limit(1024)]);

        let mut b = MultiBuffer::new();
        b.push(Buffer::from_bytes(b"abcd"));
        writer.write_multi_buffer(b).await.unwrap();

        writer.close().await.unwrap();

        let rb = reader.read_multi_buffer().await.unwrap();
        assert_eq!(rb.to_vec(), b"abcd");

        let err = reader.read_multi_buffer().await.unwrap_err();
        assert!(matches!(err, Error::Eof));
    }

    #[tokio::test]
    async fn test_pipe_discard_overflow() {
        let (reader, writer) = new_with_options(vec![with_size_limit(4), discard_overflow()]);

        let mut b1 = MultiBuffer::new();
        b1.push(Buffer::from_bytes(b"1234"));
        writer.write_multi_buffer(b1).await.unwrap();

        // Writing beyond limit with discard_overflow should succeed silently
        let mut b2 = MultiBuffer::new();
        b2.push(Buffer::from_bytes(b"5678"));
        writer.write_multi_buffer(b2).await.unwrap();

        writer.close().await.unwrap();

        let rb = reader.read_multi_buffer().await.unwrap();
        assert_eq!(rb.to_vec(), b"1234");
    }

    #[tokio::test]
    async fn test_pipe_error_injection_and_recovery() {
        let (reader, _) = new_pipe(PipeOption::new());

        reader
            .return_an_error(Error::Protocol("custom injection".into()))
            .await;

        let recovered = reader.recover().await;
        assert!(recovered.is_some());

        // Error should no longer be pending after recover
        let recovered_again = reader.recover().await;
        assert!(recovered_again.is_none());
    }
}
