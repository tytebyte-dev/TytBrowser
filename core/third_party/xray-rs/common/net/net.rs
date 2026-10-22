// Module: common\net\net.rs
// 1:1 Rust implementation corresponding to Go common\net\net.go

use std::net::IpAddr;
use std::time::Duration;

pub const CONN_IDLE_TIMEOUT: Duration = Duration::from_secs(300);
pub const QUICGO_H3_KEEP_ALIVE_PERIOD: Duration = Duration::from_secs(10);
pub const CHROME_H2_KEEP_ALIVE_PERIOD: Duration = Duration::from_secs(45);

pub fn is_local(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => v4.is_loopback() || v4.is_unspecified() || v4.is_link_local(),
        IpAddr::V6(v6) => v6.is_loopback() || v6.is_unspecified(),
    }
}
