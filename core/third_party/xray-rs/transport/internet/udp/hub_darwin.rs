// Module: transport\internet\udp\hub_darwin.rs
// 1:1 Rust implementation corresponding to Go transport\internet\udp\hub_darwin.go

use crate::common::net::Destination;
use std::net::SocketAddr;
use tokio::net::UdpSocket;

pub const SO_REUSEPORT: bool = true;

pub fn retrieve_original_dest(_oob: &[u8]) -> Option<Destination> {
    None
}

pub async fn read_udp_msg(
    conn: &UdpSocket,
    payload: &mut [u8],
    _oob: &mut [u8],
) -> std::io::Result<(usize, usize, usize, Option<SocketAddr>)> {
    let (n, addr) = conn.recv_from(payload).await?;
    Ok((n, 0, 0, Some(addr)))
}
