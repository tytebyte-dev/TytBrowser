use crate::common::buf::buffer::Buffer;

#[derive(Debug, Clone, Default)]
pub struct MultiBuffer {
    buffers: Vec<Buffer>,
}

impl MultiBuffer {
    pub fn new() -> Self {
        Self {
            buffers: Vec::new(),
        }
    }

    pub fn from_bytes(mut data: &[u8]) -> Self {
        let mut mb = Self::new();
        while !data.is_empty() {
            let mut buf = Buffer::new();
            let n = buf.write(data).unwrap();
            data = &data[n..];
            mb.buffers.push(buf);
        }
        mb
    }

    pub fn push(&mut self, buf: Buffer) {
        self.buffers.push(buf);
    }

    pub fn append(&mut self, other: &mut MultiBuffer) {
        self.buffers.append(&mut other.buffers);
    }

    pub fn clear(&mut self) {
        self.buffers.clear();
    }

    pub fn buffers(&self) -> &[Buffer] {
        &self.buffers
    }

    pub fn buffers_mut(&mut self) -> &mut [Buffer] {
        &mut self.buffers
    }

    pub fn into_buffers(self) -> Vec<Buffer> {
        self.buffers
    }

    pub fn len(&self) -> usize {
        self.buffers.iter().map(|b| b.len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.len());
        for b in &self.buffers {
            out.extend_from_slice(b.as_slice());
        }
        out
    }

    pub fn append_bytes(&mut self, mut src: &[u8]) {
        if let Some(last) = self.buffers.last_mut()
            && !last.is_full()
            && let Ok(n) = last.write(src)
        {
            src = &src[n..];
        }
        while !src.is_empty() {
            let mut b = Buffer::new();
            let n = b.write(src).unwrap_or(0);
            if n == 0 {
                break;
            }
            src = &src[n..];
            self.buffers.push(b);
        }
    }

    pub fn read_bytes(&mut self, mut dst: &mut [u8]) -> usize {
        let mut total_read = 0;
        let mut i = 0;
        while i < self.buffers.len() && !dst.is_empty() {
            let n = self.buffers[i].read(dst);
            total_read += n;
            dst = &mut dst[n..];
            if self.buffers[i].is_empty() {
                self.buffers.remove(i);
            } else {
                i += 1;
            }
        }
        total_read
    }
}
