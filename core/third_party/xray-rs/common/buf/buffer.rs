use crate::common::errors::{Error, Result};
use crate::common::net::Destination;

pub const DEFAULT_BUFFER_SIZE: usize = 8192;

#[derive(Debug, Clone)]
pub struct Buffer {
    data: Vec<u8>,
    start: usize,
    end: usize,
    pub udp: Option<Destination>,
}

impl Buffer {
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_BUFFER_SIZE)
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            data: vec![0u8; cap],
            start: 0,
            end: 0,
            udp: None,
        }
    }

    pub fn from_bytes(slice: &[u8]) -> Self {
        let mut b = Self::with_capacity(std::cmp::max(slice.len(), DEFAULT_BUFFER_SIZE));
        b.write(slice).unwrap();
        b
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn capacity(&self) -> usize {
        self.data.len()
    }

    pub fn remaining_capacity(&self) -> usize {
        self.data.len() - self.end
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    pub fn is_full(&self) -> bool {
        self.end == self.data.len()
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data[self.start..self.end]
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data[self.start..self.end]
    }

    pub fn advance(&mut self, n: usize) {
        self.start = std::cmp::min(self.start + n, self.end);
    }

    pub fn write(&mut self, src: &[u8]) -> Result<usize> {
        let available = self.remaining_capacity();
        let to_write = std::cmp::min(available, src.len());
        if to_write == 0 && !src.is_empty() {
            return Err(Error::BufferOverflow);
        }
        self.data[self.end..self.end + to_write].copy_from_slice(&src[..to_write]);
        self.end += to_write;
        Ok(to_write)
    }

    pub fn read(&mut self, dst: &mut [u8]) -> usize {
        let available = self.len();
        let to_read = std::cmp::min(available, dst.len());
        dst[..to_read].copy_from_slice(&self.data[self.start..self.start + to_read]);
        self.start += to_read;
        to_read
    }

    pub fn clear(&mut self) {
        self.start = 0;
        self.end = 0;
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for Buffer {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl Eq for Buffer {}

impl PartialEq<&[u8]> for Buffer {
    fn eq(&self, other: &&[u8]) -> bool {
        self.as_slice() == *other
    }
}

impl<const N: usize> PartialEq<&[u8; N]> for Buffer {
    fn eq(&self, other: &&[u8; N]) -> bool {
        self.as_slice() == &other[..]
    }
}
