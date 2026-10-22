// Module: proxy\tun\tun_windows.rs
// Native Wintun device implementation using pure Rust wintun-rs (embedded driver, no wintun.dll required)

#[cfg(windows)]
pub mod device {
    use std::io;
    use std::pin::Pin;
    use std::sync::Arc;
    use std::task::{Context, Poll};
    use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
    use tokio_util::sync::CancellationToken;
    use tracing::debug;
    use wintun::{Adapter, Session};

    pub struct WintunAsyncDevice {
        _adapter: Adapter,
        session: Arc<Session>,
        read_rx: tokio::sync::mpsc::Receiver<Vec<u8>>,
        cancel: CancellationToken,
    }

    impl WintunAsyncDevice {
        pub fn create(name: &str, guid: u128, ring_capacity: u32) -> io::Result<Self> {
            let adapter =
                Adapter::open(name).or_else(|_| Adapter::create(name, "Xray", Some(guid)))?;
            let session = Arc::new(adapter.start_session(ring_capacity)?);

            let (read_tx, read_rx) = tokio::sync::mpsc::channel(2048);
            let cancel = CancellationToken::new();

            let session_rx = session.clone();
            let cancel_rx = cancel.clone();

            std::thread::Builder::new()
                .name("wintun-recv".into())
                .spawn(move || {
                    while !cancel_rx.is_cancelled() {
                        let mut had_packet = false;
                        while let Ok(Some(pkt)) = session_rx.receive_packet() {
                            had_packet = true;
                            if read_tx.blocking_send(pkt.to_vec()).is_err() {
                                return;
                            }
                        }
                        if !had_packet {
                            session_rx.wait_for_data(250);
                        }
                    }
                })?;

            Ok(Self {
                _adapter: adapter,
                session,
                read_rx,
                cancel,
            })
        }
    }

    impl AsyncRead for WintunAsyncDevice {
        fn poll_read(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> Poll<io::Result<()>> {
            match self.read_rx.poll_recv(cx) {
                Poll::Ready(Some(packet)) => {
                    let n = packet.len().min(buf.remaining());
                    buf.put_slice(&packet[..n]);
                    Poll::Ready(Ok(()))
                }
                Poll::Ready(None) => Poll::Ready(Ok(())),
                Poll::Pending => Poll::Pending,
            }
        }
    }

    impl AsyncWrite for WintunAsyncDevice {
        fn poll_write(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<io::Result<usize>> {
            match self.session.allocate_send_packet(buf.len() as u32) {
                Ok(mut send_pkt) => {
                    send_pkt.copy_from_slice(buf);
                    self.session.send_packet(send_pkt);
                    Poll::Ready(Ok(buf.len()))
                }
                Err(e) => {
                    debug!("wintun allocate_send_packet error: {}", e);
                    Poll::Ready(Err(e))
                }
            }
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            Poll::Ready(Ok(()))
        }

        fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            self.cancel.cancel();
            Poll::Ready(Ok(()))
        }
    }

    impl Drop for WintunAsyncDevice {
        fn drop(&mut self) {
            self.cancel.cancel();
        }
    }
}

#[cfg(windows)]
pub use device::WintunAsyncDevice;
pub use super::runner::TunRunner as WindowsTunDevice;

