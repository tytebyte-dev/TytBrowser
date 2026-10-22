// Module: transport\internet\splithttp\upload_queue_test.rs
// 1:1 Rust unit test suite corresponding to Go transport\internet\splithttp\upload_queue_test.go

#[cfg(test)]
mod tests {
    use super::super::upload_queue::UploadQueue;

    #[tokio::test]
    async fn test_upload_queue_reordering() {
        let queue = UploadQueue::new(16);

        // Push out-of-order packets: seq 2, seq 0, seq 1
        queue.push(b"world!".to_vec(), 2).await.unwrap();
        queue.push(b"hello ".to_vec(), 0).await.unwrap();
        queue.push(b"beautiful ".to_vec(), 1).await.unwrap();

        let mut read_buf = vec![0u8; 32];
        let n = queue.read(&mut read_buf).await.unwrap();

        assert_eq!(&read_buf[..n], b"hello beautiful world!");
    }

    #[tokio::test]
    async fn test_upload_queue_partial_read() {
        let queue = UploadQueue::new(8);
        queue.push(b"0123456789".to_vec(), 0).await.unwrap();

        let mut chunk1 = [0u8; 4];
        let n1 = queue.read(&mut chunk1).await.unwrap();
        assert_eq!(n1, 4);
        assert_eq!(&chunk1, b"0123");

        let mut chunk2 = [0u8; 6];
        let n2 = queue.read(&mut chunk2).await.unwrap();
        assert_eq!(n2, 6);
        assert_eq!(&chunk2, b"456789");
    }
}
