use crate::common::buf::MultiBuffer;
use crate::common::errors::{Error, Result};
use tokio::io::{AsyncRead, AsyncReadExt};

pub struct VectorReader<R> {
    inner: R,
    chunk_size: usize,
}

impl<R: AsyncRead + Unpin> VectorReader<R> {
    pub fn new(inner: R, chunk_size: usize) -> Self {
        Self {
            inner,
            chunk_size: chunk_size.max(1024),
        }
    }

    pub async fn read_multi_buffer(&mut self, max_buffers: usize) -> Result<Option<MultiBuffer>> {
        let mut mb = MultiBuffer::new();
        let mut temp = vec![0u8; self.chunk_size];

        for _ in 0..max_buffers.max(1) {
            let n = self.inner.read(&mut temp).await.map_err(Error::Io)?;
            if n == 0 {
                break;
            }
            mb.append_bytes(&temp[..n]);
            if n < self.chunk_size {
                break;
            }
        }

        if mb.is_empty() {
            Ok(None)
        } else {
            Ok(Some(mb))
        }
    }
}
