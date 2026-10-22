// Module: transport\internet\kcp\connection.rs
// 1:1 Rust implementation corresponding to Go transport\internet\kcp\connection.go

use super::receiving::{AckList, ReceivingWindow};
use super::segment::{COMMAND_TERMINATE, CmdOnlySegment, DataSegment, Segment, read_segment};
use super::sending::SendingWindow;
use crate::common::errors::{Error, Result};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Active = 0,
    ReadyToClose = 1,
    PeerClosed = 2,
    Terminating = 3,
    PeerTerminating = 4,
    Terminated = 5,
}

pub fn now_millis() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u32
}

pub struct RoundTripInfo {
    pub variation: u32,
    pub srtt: u32,
    pub rto: u32,
    pub min_rtt: u32,
    pub updated_timestamp: u32,
}

impl RoundTripInfo {
    pub fn new() -> Self {
        Self {
            variation: 0,
            srtt: 0,
            rto: 200,
            min_rtt: 20,
            updated_timestamp: 0,
        }
    }

    pub fn update(&mut self, rtt: u32, current: u32) {
        if rtt > 0x7FFFFFFF {
            return;
        }

        // RFC 6298 RTO algorithm
        if self.srtt == 0 {
            self.srtt = rtt;
            self.variation = rtt / 2;
        } else {
            let delta = rtt.abs_diff(self.srtt);
            self.variation = (3 * self.variation + delta) / 4;
            self.srtt = (7 * self.srtt + rtt) / 8;
            if self.srtt < self.min_rtt {
                self.srtt = self.min_rtt;
            }
        }

        let mut rto = if self.min_rtt < 4 * self.variation {
            self.srtt + 4 * self.variation
        } else {
            self.srtt + self.variation
        };

        if rto > 10000 {
            rto = 10000;
        }
        self.rto = (rto * 5 / 4).max(self.min_rtt);
        self.updated_timestamp = current;
    }
}

impl Default for RoundTripInfo {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct KcpConnection {
    pub conv: u16,
    pub socket: Arc<UdpSocket>,
    pub remote_addr: Option<SocketAddr>,
    pub state: Arc<Mutex<State>>,
    pub rtt_info: Arc<Mutex<RoundTripInfo>>,
    pub send_window: Arc<Mutex<SendingWindow>>,
    pub recv_window: Arc<Mutex<ReceivingWindow>>,
    pub ack_list: Arc<Mutex<AckList>>,
    pub next_send_num: Arc<AtomicU32>,
    pub received_stream_buf: Arc<Mutex<Vec<u8>>>,
    pub mss: usize,
}

impl KcpConnection {
    pub fn new(conv: u16, socket: Arc<UdpSocket>) -> Self {
        let remote_addr = socket.peer_addr().ok();
        Self::with_remote_addr(conv, socket, remote_addr)
    }

    pub fn with_remote_addr(
        conv: u16,
        socket: Arc<UdpSocket>,
        remote_addr: Option<SocketAddr>,
    ) -> Self {
        Self {
            conv,
            socket,
            remote_addr,
            state: Arc::new(Mutex::new(State::Active)),
            rtt_info: Arc::new(Mutex::new(RoundTripInfo::new())),
            send_window: Arc::new(Mutex::new(SendingWindow::new())),
            recv_window: Arc::new(Mutex::new(ReceivingWindow::new())),
            ack_list: Arc::new(Mutex::new(AckList::new())),
            next_send_num: Arc::new(AtomicU32::new(0)),
            received_stream_buf: Arc::new(Mutex::new(Vec::with_capacity(65535))),
            mss: 1400,
        }
    }

    pub fn conv(&self) -> u16 {
        self.conv
    }

    async fn send_packet(&self, buf: &[u8]) -> Result<usize> {
        if self.socket.peer_addr().is_ok() {
            Ok(self.socket.send(buf).await?)
        } else if let Some(addr) = self.remote_addr {
            Ok(self.socket.send_to(buf, addr).await?)
        } else {
            Err(Error::Protocol(
                "missing destination address for kcp packet".into(),
            ))
        }
    }

    pub async fn is_closed(&self) -> bool {
        let state = self.state.lock().await;
        *state == State::Terminated || *state == State::Terminating
    }

    pub async fn handle_incoming_segment(&self, raw: &[u8]) -> Result<()> {
        self.input_packet(raw).await
    }

    pub async fn input_packet(&self, mut raw: &[u8]) -> Result<()> {
        let current = now_millis();
        while !raw.is_empty() {
            let (seg, remaining) = match read_segment(raw) {
                Some((s, r)) => (s, r),
                None => break,
            };
            raw = remaining;

            if seg.conversation() != self.conv {
                continue;
            }

            match seg {
                Segment::Data(data_seg) => {
                    let mut r_wnd = self.recv_window.lock().await;
                    let mut acks = self.ack_list.lock().await;
                    acks.add(data_seg.number, data_seg.timestamp);

                    let number = data_seg.number;
                    r_wnd.set(data_seg);

                    let mut stream = self.received_stream_buf.lock().await;
                    r_wnd.drain_in_order(&mut stream);

                    let mut s_wnd = self.send_window.lock().await;
                    s_wnd.handle_fast_ack(number, 200);
                }
                Segment::Ack(ack_seg) => {
                    let mut s_wnd = self.send_window.lock().await;
                    s_wnd.clear(ack_seg.receiving_next);

                    let mut rtt = self.rtt_info.lock().await;
                    for &num in &ack_seg.number_list {
                        if let Some(measured_rtt) = s_wnd.acknowledge(num, current) {
                            rtt.update(measured_rtt, current);
                        }
                    }
                }
                Segment::CmdOnly(cmd_seg) => {
                    if cmd_seg.cmd == COMMAND_TERMINATE {
                        let mut state = self.state.lock().await;
                        *state = State::PeerClosed;
                    }
                }
            }
        }

        // Flush any pending ACKs immediately
        self.flush_acks().await?;
        Ok(())
    }

    pub async fn send_data(&self, payload: &[u8]) -> Result<()> {
        if self.is_closed().await {
            return Err(Error::Closed);
        }

        let mut offset = 0;
        let mut s_wnd = self.send_window.lock().await;

        while offset < payload.len() {
            let take = (payload.len() - offset).min(self.mss);
            let chunk = payload[offset..offset + take].to_vec();
            offset += take;

            let num = self.next_send_num.fetch_add(1, Ordering::SeqCst);
            let mut seg = DataSegment::new(self.conv, num, chunk);
            seg.timestamp = now_millis();
            s_wnd.push(seg);
        }

        drop(s_wnd);
        self.flush().await?;
        Ok(())
    }

    pub async fn flush_acks(&self) -> Result<()> {
        let current = now_millis();
        let mut acks = self.ack_list.lock().await;
        let r_wnd = self.recv_window.lock().await;

        if let Some(ack_seg) = acks.flush(self.conv, 256, r_wnd.next_number, current) {
            let mut buf = vec![0u8; ack_seg.byte_size()];
            let len = ack_seg.serialize(&mut buf)?;
            self.send_packet(&buf[..len]).await?;
        }
        Ok(())
    }

    pub async fn flush(&self) -> Result<()> {
        let current = now_millis();
        let rto = {
            let rtt = self.rtt_info.lock().await;
            rtt.rto
        };

        let mut to_send = Vec::new();
        {
            let mut s_wnd = self.send_window.lock().await;
            s_wnd.flush(current, rto, 128 * 1024, &mut to_send);
        }

        for seg in to_send {
            let mut buf = vec![0u8; seg.byte_size()];
            let len = seg.serialize(&mut buf)?;
            self.send_packet(&buf[..len]).await?;
        }

        self.flush_acks().await?;
        Ok(())
    }

    pub async fn recv(&self, buf: &mut [u8]) -> Result<usize> {
        let mut stream = self.received_stream_buf.lock().await;
        if stream.is_empty() {
            return Ok(0);
        }

        let n = buf.len().min(stream.len());
        buf[..n].copy_from_slice(&stream[..n]);
        stream.drain(..n);
        Ok(n)
    }

    pub async fn close(&self) -> Result<()> {
        let mut state = self.state.lock().await;
        *state = State::Terminating;

        let cmd = CmdOnlySegment::new(self.conv, COMMAND_TERMINATE);
        let mut buf = vec![0u8; cmd.byte_size()];
        let len = cmd.serialize(&mut buf)?;
        let _ = self.send_packet(&buf[..len]).await;
        *state = State::Terminated;
        Ok(())
    }
}
