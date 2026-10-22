// Module: common\mux\writer.rs
// 1:1 Rust implementation corresponding to Go common\mux\writer.go

use crate::common::errors::Result;
use crate::common::mux::frame::{Frame, FrameMetadata, OPTION_DATA, SessionStatus};
use crate::common::net::Destination;
use tokio::io::{AsyncWrite, AsyncWriteExt};

pub struct Writer<W> {
    dest: Option<Destination>,
    writer: W,
    id: u16,
    followup: bool,
    global_id: [u8; 8],
}

impl<W: AsyncWrite + Unpin> Writer<W> {
    pub fn new(id: u16, dest: Destination, writer: W) -> Self {
        Self {
            dest: Some(dest),
            writer,
            id,
            followup: false,
            global_id: [0u8; 8],
        }
    }

    pub fn new_response_writer(id: u16, writer: W) -> Self {
        Self {
            dest: None,
            writer,
            id,
            followup: true,
            global_id: [0u8; 8],
        }
    }

    pub fn set_global_id(&mut self, id: [u8; 8]) {
        self.global_id = id;
    }

    fn get_next_frame_meta(&mut self) -> FrameMetadata {
        let mut meta = FrameMetadata::new(
            self.id,
            if self.followup {
                SessionStatus::Keep
            } else {
                self.followup = true;
                SessionStatus::New
            },
        );
        meta.target = self.dest.clone();
        meta.global_id = self.global_id;
        meta
    }

    pub async fn write_meta_only(&mut self) -> Result<()> {
        let meta = self.get_next_frame_meta();
        let mut buf = Vec::new();
        meta.write_to(&mut buf)?;
        self.writer.write_all(&buf).await?;
        self.writer.flush().await?;
        Ok(())
    }

    pub async fn write_data(&mut self, data: &[u8]) -> Result<()> {
        let mut meta = self.get_next_frame_meta();
        meta.option |= OPTION_DATA;

        let mut buf = Vec::new();
        meta.write_to(&mut buf)?;
        buf.extend_from_slice(&(data.len() as u16).to_be_bytes());
        buf.extend_from_slice(data);

        self.writer.write_all(&buf).await?;
        self.writer.flush().await?;
        Ok(())
    }

    pub async fn close(&mut self) -> Result<()> {
        let mut meta = FrameMetadata::new(self.id, SessionStatus::End);
        meta.target = self.dest.clone();
        let mut buf = Vec::new();
        meta.write_to(&mut buf)?;
        self.writer.write_all(&buf).await?;
        self.writer.flush().await?;
        Ok(())
    }
}

/// FrameWriter writes Frames to an underlying async writer.
pub struct FrameWriter<W> {
    writer: W,
}

impl<W: AsyncWrite + Unpin> FrameWriter<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    pub async fn write_frame(&mut self, frame: &Frame) -> Result<()> {
        frame.write_to(&mut self.writer).await
    }
}
