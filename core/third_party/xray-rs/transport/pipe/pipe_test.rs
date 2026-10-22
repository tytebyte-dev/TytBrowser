// Module: transport\\pipe\\pipe_test.rs
// 1:1 Rust unit test suite corresponding to Go transport\\pipe\\pipe_test.go

#[cfg(test)]
mod tests {
    use super::super::impl_::PipeOption;
    use super::super::pipe::new_pipe;
    use crate::common::buf::Buffer;

    #[tokio::test]
    async fn test_pipe_read_write() {
        let (r, w) = new_pipe(PipeOption::new());
        let mut buf = Buffer::new();
        buf.write(b"test payload").unwrap();
        w.write_buffer(buf).await.unwrap();

        let read_mb = r.read_multi_buffer().await.unwrap();
        assert_eq!(read_mb.len(), 12);
        assert_eq!(read_mb.to_vec(), b"test payload");
    }
}
