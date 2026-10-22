// Module: proxy\mod.rs

pub mod blackhole;
pub mod dns;
pub mod dokodemo;
pub mod freedom;
pub mod http;
pub mod hysteria;
pub mod loopback;
pub mod mixed;
pub mod proxy;
pub mod shadowsocks;
pub mod shadowsocks_2022;
pub mod socks;
pub mod trojan;
pub mod tun;
pub mod vless;
pub mod vmess;
pub mod wireguard;

#[cfg(test)]
pub mod proxy_test;

pub use dns::DnsOutbound;
pub use loopback::LoopbackOutbound;
pub use proxy::{
    COMMAND_PADDING_CONTINUE, COMMAND_PADDING_DIRECT, COMMAND_PADDING_END, DefaultUserManager,
    Inbound, InboundState, Outbound, OutboundState, PROXY_VERSION, TLS_APPLICATION_DATA_START,
    TLS_CLIENT_HANDSHAKE_START, TLS_HANDSHAKE_TYPE_CLIENT_HELLO, TLS_HANDSHAKE_TYPE_SERVER_HELLO,
    TLS_SERVER_HANDSHAKE_START, TLS13_CIPHER_SUITE_DIC, TLS13_SUPPORTED_VERSIONS, TrafficState,
    UserManager, is_complete_record, xtls_filter_tls, xtls_padding, xtls_unpadding,
};
pub use tun::{TunConfig, TunHandler};
