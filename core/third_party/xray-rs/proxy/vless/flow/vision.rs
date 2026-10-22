// Module: proxy\vless\flow\vision.rs
// 1:1 Rust implementation corresponding to Go proxy\vless\flow\vision.go & proxy\vless\encoding\addons.go

use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, ReadBuf};

use crate::common::errors::Result;
use crate::proxy::proxy::{
    COMMAND_PADDING_CONTINUE, COMMAND_PADDING_DIRECT, COMMAND_PADDING_END, TrafficState,
    xtls_filter_tls, xtls_padding, xtls_unpadding,
};

pub const FLOW_VISION: &str = "xtls-rprx-vision";
pub const FLOW_VISION_UDP443: &str = "xtls-rprx-vision-udp443";

#[derive(Debug, Clone)]
pub struct VisionContext {
    pub is_tls: Arc<AtomicBool>,
    pub direct_copy: Arc<AtomicBool>,
    pub traffic_state: Arc<Mutex<TrafficState>>,
    pub is_uplink: bool,
}

impl VisionContext {
    pub fn new(user_uuid: Vec<u8>, is_uplink: bool) -> Self {
        Self {
            is_tls: Arc::new(AtomicBool::new(false)),
            direct_copy: Arc::new(AtomicBool::new(false)),
            traffic_state: Arc::new(Mutex::new(TrafficState::new(&user_uuid))),
            is_uplink,
        }
    }

    pub fn is_direct_copy(&self) -> bool {
        self.direct_copy.load(Ordering::SeqCst)
    }

    pub fn set_direct_copy(&self, val: bool) {
        self.direct_copy.store(val, Ordering::SeqCst);
    }

    pub fn is_tls(&self) -> bool {
        self.is_tls.load(Ordering::SeqCst)
    }

    pub fn set_tls(&self, val: bool) {
        self.is_tls.store(val, Ordering::SeqCst);
    }
}

impl Default for VisionContext {
    fn default() -> Self {
        Self::new(vec![0u8; 16], true)
    }
}

pub struct VisionFilter;

impl VisionFilter {
    pub fn inspect_tls_record_length(buf: &[u8]) -> Option<usize> {
        if buf.len() < 5 {
            return None;
        }
        let length = u16::from_be_bytes([buf[3], buf[4]]) as usize;
        Some(length + 5)
    }

    pub async fn write_padded<W: AsyncWrite + Unpin>(
        writer: &mut W,
        data: &[u8],
        padding_len: usize,
    ) -> Result<()> {
        writer.write_all(data).await?;
        if padding_len > 0 {
            let padding = vec![0u8; padding_len];
            writer.write_all(&padding).await?;
        }
        writer.flush().await?;
        Ok(())
    }
}

pub struct VisionStream<S> {
    inner: S,
    pub context: VisionContext,
    is_inbound: bool,
    read_buf: Vec<u8>,
    write_buf: Vec<u8>,
}

impl<S> VisionStream<S> {
    pub fn new(inner: S, context: VisionContext, is_inbound: bool) -> Self {
        Self {
            inner,
            context,
            is_inbound,
            read_buf: Vec::with_capacity(4096),
            write_buf: Vec::with_capacity(4096),
        }
    }

    pub fn into_inner(self) -> S {
        self.inner
    }

    pub fn get_ref(&self) -> &S {
        &self.inner
    }

    pub fn get_mut(&mut self) -> &mut S {
        &mut self.inner
    }
}

impl<S: AsyncRead + Unpin> AsyncRead for VisionStream<S> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        // If remaining decrypted/unpadded payload is in buffer, drain it first
        if !self.read_buf.is_empty() {
            let to_write = std::cmp::min(buf.remaining(), self.read_buf.len());
            buf.put_slice(&self.read_buf[..to_write]);
            self.read_buf.drain(..to_write);
            return Poll::Ready(Ok(()));
        }

        // If direct copy mode has been established, bypass XTLS Vision framing completely
        if self.context.is_direct_copy() {
            return Pin::new(&mut self.inner).poll_read(cx, buf);
        }

        let mut raw = [0u8; 4096];
        let mut read_buf = ReadBuf::new(&mut raw);
        match Pin::new(&mut self.inner).poll_read(cx, &mut read_buf) {
            Poll::Ready(Ok(())) => {
                let filled = read_buf.filled();
                if filled.is_empty() {
                    return Poll::Ready(Ok(()));
                }

                let unpadded = {
                    let mut state = self.context.traffic_state.lock().unwrap();
                    let is_uplink = self.is_inbound;
                    let data = xtls_unpadding(filled, &mut state, is_uplink);

                    // Check if direct copy triggered
                    let should_direct = if is_uplink {
                        state.inbound.current_command == (COMMAND_PADDING_DIRECT as i32)
                            || state.inbound.uplink_reader_direct_copy
                    } else {
                        state.outbound.current_command == (COMMAND_PADDING_DIRECT as i32)
                            || state.outbound.downlink_reader_direct_copy
                    };

                    if should_direct {
                        self.context.set_direct_copy(true);
                    }

                    data
                };

                let to_write = std::cmp::min(buf.remaining(), unpadded.len());
                buf.put_slice(&unpadded[..to_write]);
                if to_write < unpadded.len() {
                    self.read_buf.extend_from_slice(&unpadded[to_write..]);
                }
                Poll::Ready(Ok(()))
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<S: AsyncWrite + Unpin> AsyncWrite for VisionStream<S> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        let this = self.get_mut();

        // If direct copy mode is on, write directly with 0 overhead
        if this.context.is_direct_copy() {
            return Pin::new(&mut this.inner).poll_write(cx, buf);
        }

        // First flush any previously buffered write data
        while !this.write_buf.is_empty() {
            match Pin::new(&mut this.inner).poll_write(cx, &this.write_buf) {
                Poll::Ready(Ok(n)) => {
                    this.write_buf.drain(..n);
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }

        let padded_frame = {
            let mut state = this.context.traffic_state.lock().unwrap();
            xtls_filter_tls(buf, &mut state);

            let command = if state.is_tls && state.is_tls12_or_above {
                COMMAND_PADDING_DIRECT
            } else if state.number_of_packet_to_filter <= 0 {
                COMMAND_PADDING_END
            } else {
                COMMAND_PADDING_CONTINUE
            };

            let uuid = state.user_uuid.clone();
            let frame = xtls_padding(Some(buf), command, Some(&uuid), true, None);

            if command == COMMAND_PADDING_DIRECT {
                this.context.set_direct_copy(true);
            }

            frame
        };

        match Pin::new(&mut this.inner).poll_write(cx, &padded_frame) {
            Poll::Ready(Ok(n)) => {
                if n < padded_frame.len() {
                    this.write_buf.extend_from_slice(&padded_frame[n..]);
                }
                Poll::Ready(Ok(buf.len()))
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        let this = self.get_mut();
        while !this.write_buf.is_empty() {
            match Pin::new(&mut this.inner).poll_write(cx, &this.write_buf) {
                Poll::Ready(Ok(n)) => {
                    this.write_buf.drain(..n);
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }
        Pin::new(&mut this.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}
