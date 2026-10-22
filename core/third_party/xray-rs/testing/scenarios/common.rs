// Module: testing\scenarios\common.rs
// Test framework common environment, helpers and protocols connectors

use std::sync::atomic::{AtomicU16, Ordering};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::task::JoinHandle;

use crate::common::errors::{Error, Result};
use crate::core::Instance;
use crate::infra::conf::Config;

static NEXT_PORT: AtomicU16 = AtomicU16::new(24500);

pub async fn pick_port() -> u16 {
    // Try to get dynamic ephemeral port from OS
    if let Ok(listener) = tokio::net::TcpListener::bind("127.0.0.1:0").await
        && let Ok(addr) = listener.local_addr()
    {
        drop(listener);
        return addr.port();
    }
    NEXT_PORT.fetch_add(1, Ordering::SeqCst)
}

pub fn random_payload(size: usize) -> Vec<u8> {
    use rand::RngCore;
    let mut buf = vec![0u8; size];
    rand::thread_rng().fill_bytes(&mut buf);
    buf
}

pub fn xor(data: &[u8], key: u8) -> Vec<u8> {
    data.iter().map(|b| b ^ key).collect()
}

pub struct TestEnvironment {
    tasks: Vec<JoinHandle<()>>,
}

impl TestEnvironment {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub async fn start_node(&mut self, config: Config) -> Result<()> {
        let instance = Instance::from_config(config)?;
        let mut handles = instance.start().await?;
        self.tasks.append(&mut handles);
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok(())
    }

    pub fn close_all(&mut self) {
        for handle in self.tasks.drain(..) {
            handle.abort();
        }
    }
}

impl Default for TestEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        self.close_all();
    }
}

/// SOCKS5 No-Auth Connect Helper
pub async fn socks5_connect(
    proxy_port: u16,
    target_host: &str,
    target_port: u16,
) -> Result<TcpStream> {
    let mut stream = TcpStream::connect(("127.0.0.1", proxy_port)).await?;
    stream.set_nodelay(true)?;

    // 1. Handshake greeting: NO_AUTH (0x00)
    stream.write_all(&[0x05, 0x01, 0x00]).await?;
    let mut resp = [0u8; 2];
    stream.read_exact(&mut resp).await?;
    if resp[0] != 0x05 || resp[1] != 0x00 {
        return Err(Error::Protocol(format!(
            "SOCKS5 greeting failed: {:?}",
            resp
        )));
    }

    // 2. CONNECT request
    let mut req = Vec::with_capacity(64);
    if let Ok(ip) = target_host.parse::<std::net::Ipv4Addr>() {
        req.extend_from_slice(&[0x05, 0x01, 0x00, 0x01]);
        req.extend_from_slice(&ip.octets());
    } else if let Ok(ip) = target_host.parse::<std::net::Ipv6Addr>() {
        req.extend_from_slice(&[0x05, 0x01, 0x00, 0x04]);
        req.extend_from_slice(&ip.octets());
    } else {
        req.extend_from_slice(&[0x05, 0x01, 0x00, 0x03]);
        req.push(target_host.len() as u8);
        req.extend_from_slice(target_host.as_bytes());
    }
    req.extend_from_slice(&target_port.to_be_bytes());
    stream.write_all(&req).await?;

    let mut reply = [0u8; 4];
    stream.read_exact(&mut reply).await?;
    if reply[1] != 0x00 {
        return Err(Error::Protocol(format!(
            "SOCKS5 connect error code: {}",
            reply[1]
        )));
    }

    // Skip bound address
    match reply[3] {
        0x01 => {
            let mut addr = [0u8; 4 + 2];
            stream.read_exact(&mut addr).await?;
        }
        0x04 => {
            let mut addr = [0u8; 16 + 2];
            stream.read_exact(&mut addr).await?;
        }
        0x03 => {
            let mut len = [0u8; 1];
            stream.read_exact(&mut len).await?;
            let mut domain = vec![0u8; len[0] as usize + 2];
            stream.read_exact(&mut domain).await?;
        }
        _ => {
            return Err(Error::Protocol(
                "Unknown address type in SOCKS5 reply".into(),
            ));
        }
    }

    Ok(stream)
}

/// SOCKS5 Username/Password Auth Connect Helper
pub async fn socks5_connect_with_auth(
    proxy_port: u16,
    target_host: &str,
    target_port: u16,
    username: &str,
    password: &str,
) -> Result<TcpStream> {
    let mut stream = TcpStream::connect(("127.0.0.1", proxy_port)).await?;
    stream.set_nodelay(true)?;

    // 1. Handshake greeting: USER_PASS (0x02)
    stream.write_all(&[0x05, 0x01, 0x02]).await?;
    let mut resp = [0u8; 2];
    stream.read_exact(&mut resp).await?;
    if resp[0] != 0x05 || resp[1] != 0x02 {
        return Err(Error::Protocol(format!(
            "SOCKS5 auth negotiation failed: {:?}",
            resp
        )));
    }

    // 2. Send auth subnegotiation
    let mut auth_req = Vec::new();
    auth_req.push(0x01); // Auth version 1
    auth_req.push(username.len() as u8);
    auth_req.extend_from_slice(username.as_bytes());
    auth_req.push(password.len() as u8);
    auth_req.extend_from_slice(password.as_bytes());
    stream.write_all(&auth_req).await?;

    let mut auth_resp = [0u8; 2];
    stream.read_exact(&mut auth_resp).await?;
    if auth_resp[1] != 0x00 {
        return Err(Error::AuthFailed(
            "SOCKS5 user/pass authentication failed".into(),
        ));
    }

    // 3. CONNECT request
    let mut req = Vec::with_capacity(64);
    if let Ok(ip) = target_host.parse::<std::net::Ipv4Addr>() {
        req.extend_from_slice(&[0x05, 0x01, 0x00, 0x01]);
        req.extend_from_slice(&ip.octets());
    } else {
        req.extend_from_slice(&[0x05, 0x01, 0x00, 0x03]);
        req.push(target_host.len() as u8);
        req.extend_from_slice(target_host.as_bytes());
    }
    req.extend_from_slice(&target_port.to_be_bytes());
    stream.write_all(&req).await?;

    let mut reply = [0u8; 4];
    stream.read_exact(&mut reply).await?;
    if reply[1] != 0x00 {
        return Err(Error::Protocol(format!(
            "SOCKS5 connect error code: {}",
            reply[1]
        )));
    }

    match reply[3] {
        0x01 => {
            let mut addr = [0u8; 4 + 2];
            stream.read_exact(&mut addr).await?;
        }
        0x04 => {
            let mut addr = [0u8; 16 + 2];
            stream.read_exact(&mut addr).await?;
        }
        0x03 => {
            let mut len = [0u8; 1];
            stream.read_exact(&mut len).await?;
            let mut domain = vec![0u8; len[0] as usize + 2];
            stream.read_exact(&mut domain).await?;
        }
        _ => {
            return Err(Error::Protocol(
                "Unknown address type in SOCKS5 reply".into(),
            ));
        }
    }

    Ok(stream)
}

/// HTTP CONNECT Tunnel Helper
pub async fn http_connect_tunnel(
    proxy_port: u16,
    target_host: &str,
    target_port: u16,
) -> Result<TcpStream> {
    let mut stream = TcpStream::connect(("127.0.0.1", proxy_port)).await?;
    stream.set_nodelay(true)?;

    let req = format!(
        "CONNECT {}:{} HTTP/1.1\r\nHost: {}:{}\r\n\r\n",
        target_host, target_port, target_host, target_port
    );
    stream.write_all(req.as_bytes()).await?;

    let mut buf = [0u8; 1024];
    let n = stream.read(&mut buf).await?;
    let resp = std::str::from_utf8(&buf[..n])
        .map_err(|e| Error::Protocol(format!("Invalid HTTP response: {}", e)))?;

    if !resp.contains("200") {
        return Err(Error::Protocol(format!("HTTP CONNECT failed: {}", resp)));
    }

    Ok(stream)
}

/// HTTP GET Through Proxy Helper
pub async fn http_get_proxy(proxy_port: u16, target_url: &str) -> Result<(u16, Vec<u8>)> {
    let mut stream = TcpStream::connect(("127.0.0.1", proxy_port)).await?;
    stream.set_nodelay(true)?;

    let host = target_url
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or("localhost");
    let req = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        target_url, host
    );
    stream.write_all(req.as_bytes()).await?;

    let mut resp = Vec::new();
    stream.read_to_end(&mut resp).await?;

    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut parsed_resp = httparse::Response::new(&mut headers);
    match parsed_resp.parse(&resp) {
        Ok(httparse::Status::Complete(header_len)) => {
            let status = parsed_resp.code.unwrap_or(0);
            let body = resp[header_len..].to_vec();
            Ok((status, body))
        }
        _ => Err(Error::Protocol(
            "Incomplete HTTP response from proxy".into(),
        )),
    }
}

/// Wait connection available with retries
pub async fn wait_conn_available<F, Fut>(max_attempts: usize, delay: Duration, test_fn: F) -> bool
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<()>>,
{
    for _ in 0..max_attempts {
        tokio::time::sleep(delay).await;
        if test_fn().await.is_ok() {
            return true;
        }
    }
    false
}
