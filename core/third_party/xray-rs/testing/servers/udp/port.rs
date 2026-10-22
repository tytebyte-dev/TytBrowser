// Module: testing\servers\udp\port.rs
// UDP dynamic port picker

use tokio::net::UdpSocket;

/// PickPort returns an unused UDP port in the system by temporarily binding to 127.0.0.1:0.
pub async fn pick_port() -> u16 {
    let socket = UdpSocket::bind("127.0.0.1:0")
        .await
        .expect("bind ephemeral UDP port");
    let port = socket.local_addr().expect("local addr").port();
    drop(socket);
    port
}
