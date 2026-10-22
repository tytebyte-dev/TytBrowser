// Module: transport\internet\happy_eyeballs.rs
// 1:1 Rust implementation corresponding to Go transport\internet\happy_eyeballs.go

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::sleep;

use crate::common::errors::{Error, Result};

#[derive(Debug, Clone)]
pub struct HappyEyeballsConfig {
    pub prioritize_ipv6: bool,
    pub interleave: u32,
    pub try_delay_ms: u64,
    pub max_concurrent_try: usize,
}

impl Default for HappyEyeballsConfig {
    fn default() -> Self {
        Self {
            prioritize_ipv6: false,
            interleave: 1,
            try_delay_ms: 250,
            max_concurrent_try: 4,
        }
    }
}

pub trait IntoInterleave {
    fn into_interleave(self) -> u32;
}

impl IntoInterleave for u32 {
    fn into_interleave(self) -> u32 {
        self
    }
}

impl IntoInterleave for usize {
    fn into_interleave(self) -> u32 {
        self as u32
    }
}

impl IntoInterleave for bool {
    fn into_interleave(self) -> u32 {
        if self { 1 } else { 0 }
    }
}

/// Sorts IPs according to RFC 8305.
pub fn sort_ips(
    ips: &[IpAddr],
    prioritize_ipv6: bool,
    interleave: impl IntoInterleave,
) -> Vec<IpAddr> {
    if ips.is_empty() {
        return Vec::new();
    }

    let mut ip4: Vec<IpAddr> = Vec::new();
    let mut ip6: Vec<IpAddr> = Vec::new();

    for ip in ips {
        if ip.is_ipv4() {
            ip4.push(*ip);
        } else {
            ip6.push(*ip);
        }
    }

    if ip4.is_empty() || ip6.is_empty() {
        return ips.to_vec();
    }

    let interleave = interleave.into_interleave();
    if interleave == 0 {
        if prioritize_ipv6 {
            let mut res = ip6;
            res.extend(ip4);
            return res;
        } else {
            let mut res = ip4;
            res.extend(ip6);
            return res;
        }
    }

    let mut new_ips = Vec::with_capacity(ips.len());
    let mut consume_ip4 = 0;
    let mut consume_ip6 = 0;
    let mut consume_turn = 0u32;
    let mut ip4_turn = !prioritize_ipv6;

    loop {
        if ip4_turn {
            new_ips.push(ip4[consume_ip4]);
            consume_ip4 += 1;
            if consume_ip4 == ip4.len() {
                new_ips.extend_from_slice(&ip6[consume_ip6..]);
                break;
            }
            consume_turn += 1;
            if consume_turn == interleave {
                ip4_turn = false;
                consume_turn = 0;
            }
        } else {
            new_ips.push(ip6[consume_ip6]);
            consume_ip6 += 1;
            if consume_ip6 == ip6.len() {
                new_ips.extend_from_slice(&ip4[consume_ip4..]);
                break;
            }
            consume_turn += 1;
            if consume_turn == interleave {
                ip4_turn = true;
                consume_turn = 0;
            }
        }
    }

    new_ips
}

pub async fn tcp_race_dial(
    ips: &[IpAddr],
    port: u16,
    config: &HappyEyeballsConfig,
    _domain: &str,
) -> Result<TcpStream> {
    if ips.is_empty() {
        return Err(Error::Other("at least 1 ip is required to dial".into()));
    }
    if ips.len() == 1 {
        let addr = SocketAddr::new(ips[0], port);
        return TcpStream::connect(addr).await.map_err(Error::Io);
    }

    let sorted = sort_ips(ips, config.prioritize_ipv6, config.interleave);
    let (tx, mut rx) = mpsc::channel(sorted.len());
    let cancelled = Arc::new(AtomicBool::new(false));

    let try_delay = Duration::from_millis(config.try_delay_ms);
    let max_concurrent = config.max_concurrent_try.max(1);

    let mut next_try_idx = 0;
    let mut active_num = 0;

    // Initial launch
    let tx_clone = tx.clone();
    let cancelled_clone = cancelled.clone();
    let addr = SocketAddr::new(sorted[0], port);
    tokio::spawn(async move {
        let res = TcpStream::connect(addr).await;
        if !cancelled_clone.load(Ordering::Relaxed) {
            let _ = tx_clone.send(res).await;
        }
    });
    active_num += 1;
    next_try_idx += 1;

    let mut delay_fut = Box::pin(sleep(try_delay));

    loop {
        tokio::select! {
            Some(result) = rx.recv() => {
                active_num -= 1;
                match result {
                    Ok(conn) => {
                        cancelled.store(true, Ordering::SeqCst);
                        return Ok(conn);
                    }
                    Err(e) => {
                        if next_try_idx < sorted.len() && active_num < max_concurrent {
                            let tx_clone = tx.clone();
                            let cancelled_clone = cancelled.clone();
                            let addr = SocketAddr::new(sorted[next_try_idx], port);
                            tokio::spawn(async move {
                                let res = TcpStream::connect(addr).await;
                                if !cancelled_clone.load(Ordering::Relaxed) {
                                    let _ = tx_clone.send(res).await;
                                }
                            });
                            active_num += 1;
                            next_try_idx += 1;
                            delay_fut = Box::pin(sleep(try_delay));
                        } else if active_num == 0 {
                            return Err(Error::Io(e));
                        }
                    }
                }
            }
            _ = &mut delay_fut, if next_try_idx < sorted.len() && active_num < max_concurrent => {
                let tx_clone = tx.clone();
                let cancelled_clone = cancelled.clone();
                let addr = SocketAddr::new(sorted[next_try_idx], port);
                tokio::spawn(async move {
                    let res = TcpStream::connect(addr).await;
                    if !cancelled_clone.load(Ordering::Relaxed) {
                        let _ = tx_clone.send(res).await;
                    }
                });
                active_num += 1;
                next_try_idx += 1;
                delay_fut = Box::pin(sleep(try_delay));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn test_sort_ips_rfc8305() {
        let v4_1 = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
        let v4_2 = IpAddr::V4(Ipv4Addr::new(1, 0, 0, 1));
        let v6_1 = IpAddr::V6(Ipv6Addr::new(0x2606, 0x4700, 0x4700, 0, 0, 0, 0, 0x1111));
        let v6_2 = IpAddr::V6(Ipv6Addr::new(0x2606, 0x4700, 0x4700, 0, 0, 0, 0, 0x1001));

        let ips = vec![v4_1, v4_2, v6_1, v6_2];

        // Default: prioritize IPv4, interleave 1 -> v4_1, v6_1, v4_2, v6_2
        let sorted = sort_ips(&ips, false, 1u32);
        assert_eq!(sorted, vec![v4_1, v6_1, v4_2, v6_2]);

        // Prioritize IPv6, interleave 1 -> v6_1, v4_1, v6_2, v4_2
        let sorted_v6 = sort_ips(&ips, true, 1u32);
        assert_eq!(sorted_v6, vec![v6_1, v4_1, v6_2, v4_2]);

        // Prioritize IPv4, interleave 2 -> v4_1, v4_2, v6_1, v6_2
        let sorted_int2 = sort_ips(&ips, false, 2u32);
        assert_eq!(sorted_int2, vec![v4_1, v4_2, v6_1, v6_2]);

        // Interleave false -> all v4 then v6
        let no_interleave = sort_ips(&ips, false, false);
        assert_eq!(no_interleave, vec![v4_1, v4_2, v6_1, v6_2]);
    }

    #[tokio::test]
    async fn test_tcp_race_dial_single() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let cfg = HappyEyeballsConfig::default();
        let stream = tcp_race_dial(&[IpAddr::V4(Ipv4Addr::LOCALHOST)], port, &cfg, "localhost")
            .await
            .unwrap();
        assert_eq!(stream.peer_addr().unwrap().port(), port);
    }
}
