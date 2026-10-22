use crate::transport::internet::StreamSettings;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnownProtocols {
    Http,
    Tls,
    Quic,
    Bittorrent,
    Fakedns,
}

impl KnownProtocols {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "http" => Some(KnownProtocols::Http),
            "tls" => Some(KnownProtocols::Tls),
            "quic" => Some(KnownProtocols::Quic),
            "bittorrent" => Some(KnownProtocols::Bittorrent),
            "fakedns" => Some(KnownProtocols::Fakedns),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SniffingConfig {
    pub enabled: bool,
    pub dest_override: Vec<String>,
    pub metadata_only: bool,
    pub route_only: bool,
}

#[derive(Debug, Clone, Default)]
pub struct InboundHandlerConfig {
    pub tag: String,
    pub listen: String,
    pub port: u16,
    pub protocol: String,
    pub sniffing: Option<SniffingConfig>,
    pub stream_settings: Option<StreamSettings>,
}

#[derive(Debug, Clone, Default)]
pub struct OutboundHandlerConfig {
    pub tag: String,
    pub protocol: String,
    pub stream_settings: Option<StreamSettings>,
}
