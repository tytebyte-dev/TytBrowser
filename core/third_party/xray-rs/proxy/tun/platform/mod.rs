#[cfg(target_os = "windows")]
mod win;
#[cfg(target_os = "windows")]
pub use win::must_bind_socket_on_interface;

#[cfg(not(target_os = "windows"))]
pub fn must_bind_socket_on_interface(
    _socket: &socket2::Socket,
    _iface: &crate::proxy::tun::OutboundInterface,
    _family: socket2::Domain,
) -> std::io::Result<()> {
    Ok(())
}
