// Module: transport\\link.rs
// 1:1 Rust implementation corresponding to Go transport\\link.go

use crate::transport::pipe::{Reader as PipeReader, Writer as PipeWriter};

#[derive(Clone)]
pub struct Link {
    pub reader: PipeReader,
    pub writer: PipeWriter,
}

impl Link {
    pub fn new(reader: PipeReader, writer: PipeWriter) -> Self {
        Self { reader, writer }
    }

    pub fn split(self) -> (PipeReader, PipeWriter) {
        (self.reader, self.writer)
    }

    pub fn into_parts(self) -> (PipeReader, PipeWriter) {
        (self.reader, self.writer)
    }
}
