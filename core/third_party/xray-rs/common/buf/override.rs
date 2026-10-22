// Module: common\buf\override.rs
// 1:1 Rust implementation corresponding to Go common\buf\override.go

use async_trait::async_trait;

use crate::common::buf::MultiBuffer;
use crate::common::buf::io::{Reader, Writer};
use crate::common::errors::Result;
use crate::common::net::Address;

pub struct EndpointOverrideReader<R: Reader> {
    pub reader: R,
    pub dest: Address,
    pub original_dest: Address,
}

impl<R: Reader> EndpointOverrideReader<R> {
    pub fn new(reader: R, dest: Address, original_dest: Address) -> Self {
        Self {
            reader,
            dest,
            original_dest,
        }
    }
}

#[async_trait]
impl<R: Reader + Send + Sync> Reader for EndpointOverrideReader<R> {
    async fn read_multi_buffer(&mut self) -> Result<MultiBuffer> {
        let mut mb = self.reader.read_multi_buffer().await?;
        for b in mb.buffers_mut() {
            if let Some(ref mut udp) = b.udp
                && udp.address == self.original_dest
            {
                udp.address = self.dest.clone();
            }
        }
        Ok(mb)
    }
}

pub struct EndpointOverrideWriter<W: Writer> {
    pub writer: W,
    pub dest: Address,
    pub original_dest: Address,
}

impl<W: Writer> EndpointOverrideWriter<W> {
    pub fn new(writer: W, dest: Address, original_dest: Address) -> Self {
        Self {
            writer,
            dest,
            original_dest,
        }
    }
}

#[async_trait]
impl<W: Writer + Send + Sync> Writer for EndpointOverrideWriter<W> {
    async fn write_multi_buffer(&mut self, mut mb: MultiBuffer) -> Result<()> {
        for b in mb.buffers_mut() {
            if let Some(ref mut udp) = b.udp
                && udp.address == self.dest
            {
                udp.address = self.original_dest.clone();
            }
        }
        self.writer.write_multi_buffer(mb).await
    }
}
