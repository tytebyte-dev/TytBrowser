// Socket helpers for outbound interface binding (aligned with Xray-core)
use super::platform::must_bind_socket_on_interface;
use crate::proxy::tun::OutboundInterface;

use socket2::TcpKeepalive;
use std::{net::SocketAddr, time::Duration};
use tokio::{
    net::{TcpSocket, TcpStream, UdpSocket},
    time::timeout,
};
use tracing::{debug, error, trace};

pub fn apply_tcp_options(s: &TcpStream) -> std::io::Result<()> {
    #[cfg(not(target_os = "windows"))]
    {
        let sock_ref = socket2::SockRef::from(s);
        sock_ref.set_tcp_keepalive(
            &TcpKeepalive::new()
                .with_time(Duration::from_secs(10))
                .with_interval(Duration::from_secs(1))
                .with_retries(3),
        )?;
    }
    #[cfg(target_os = "windows")]
    {
        let sock_ref = socket2::SockRef::from(s);
        sock_ref.set_tcp_keepalive(
            &TcpKeepalive::new()
                .with_time(Duration::from_secs(10))
                .with_interval(Duration::from_secs(1)),
        )?;
    }
    s.set_nodelay(true)
}

pub async fn new_tcp_stream(
    endpoint: SocketAddr,
    iface: Option<&OutboundInterface>,
) -> std::io::Result<TcpStream> {
    let (socket, family) = match endpoint {
        SocketAddr::V4(_) => (
            socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::STREAM, None)?,
            socket2::Domain::IPV4,
        ),
        SocketAddr::V6(_) => (
            socket2::Socket::new(socket2::Domain::IPV6, socket2::Type::STREAM, None)?,
            socket2::Domain::IPV6,
        ),
    };
    debug!("created tcp socket for {}", endpoint);

    if !cfg!(target_os = "android")
        && !endpoint.ip().is_loopback()
        && let Some(iface) = iface
    {
        must_bind_socket_on_interface(&socket, iface, family)?;
        trace!("tcp socket bound to interface: {socket:?}");
    }

    socket.set_keepalive(true)?;
    socket.set_nodelay(true)?;
    socket.set_nonblocking(true)?;

    timeout(
        Duration::from_secs(10),
        TcpSocket::from_std_stream(socket.into()).connect(endpoint),
    )
    .await?
}

pub async fn new_udp_socket(
    src: Option<SocketAddr>,
    iface: Option<&OutboundInterface>,
    family_hint: Option<std::net::SocketAddr>,
) -> std::io::Result<UdpSocket> {
    let (socket, family) = match (family_hint, src, iface) {
        (Some(family_hint), ..) => {
            let domain = socket2::Domain::for_address(family_hint);
            (
                socket2::Socket::new(domain, socket2::Type::DGRAM, None)?,
                domain,
            )
        }
        (None, Some(src), _) if src.is_ipv6() => (
            try_create_dualstack_socket(src, socket2::Type::DGRAM)?.0,
            socket2::Domain::IPV6,
        ),
        (None, _, Some(iface)) if iface.addr_v6.is_some() => (
            socket2::Socket::new(socket2::Domain::IPV6, socket2::Type::DGRAM, None)?,
            socket2::Domain::IPV6,
        ),
        _ => (
            socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::DGRAM, None)?,
            socket2::Domain::IPV4,
        ),
    };
    debug!("created udp socket");

    if !cfg!(target_os = "android") {
        let dst_is_loopback = family_hint.map(|a| a.ip().is_loopback()).unwrap_or(false);
        match (src, iface) {
            (_, Some(iface)) if !dst_is_loopback => {
                must_bind_socket_on_interface(&socket, iface, family).map_err(|x| {
                    error!("failed to bind socket to interface: {}", x);
                    x
                })?;
                #[cfg(target_os = "windows")]
                if let Some(addr) = src {
                    socket.bind(&socket2::SockAddr::from(addr))?;
                }

                trace!(iface = ?iface, "udp socket bound: {socket:?}");
            }
            (Some(src), _) => {
                socket.bind(&src.into())?;
                trace!(src = ?src, "udp socket bound: {socket:?}");
            }
            (None, _) => {
                #[cfg(target_os = "windows")]
                {
                    let bind_addr = match family {
                        socket2::Domain::IPV4 => "0.0.0.0:0".parse::<SocketAddr>().unwrap(),
                        socket2::Domain::IPV6 => "[::]:0".parse::<SocketAddr>().unwrap(),
                        _ => "0.0.0.0:0".parse::<SocketAddr>().unwrap(),
                    };
                    socket.bind(&socket2::SockAddr::from(bind_addr))?;
                    trace!(addr = ?bind_addr, "udp socket bound to default address on Windows: {socket:?}");
                }
                #[cfg(not(target_os = "windows"))]
                trace!("udp socket not bound to any specific address: {socket:?}");
            }
        }
    }

    socket.set_broadcast(true)?;
    socket.set_nonblocking(true)?;

    UdpSocket::from_std(socket.into())
}

pub fn try_create_dualstack_socket(
    addr: SocketAddr,
    socket_type: socket2::Type,
) -> std::io::Result<(socket2::Socket, socket2::Domain)> {
    let domain = socket2::Domain::for_address(addr);
    let socket = socket2::Socket::new(domain, socket_type, None)?;
    if domain == socket2::Domain::IPV6 {
        let _ = socket.set_only_v6(false);
    }
    Ok((socket, domain))
}
