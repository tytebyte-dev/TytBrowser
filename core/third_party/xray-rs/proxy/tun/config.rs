use crate::common::session::SniffingRequest;
use ipnet::{IpNet, Ipv4Net, Ipv6Net};

#[derive(Debug, Clone)]
pub struct TunConfig {
    pub enable: bool,
    pub name: String,
    pub device_id: String,
    pub route_all: bool,
    pub routes: Vec<IpNet>,
    pub gateway: Ipv4Net,
    pub gateway_v6: Option<Ipv6Net>,
    pub mtu: usize,
    pub so_mark: Option<u32>,
    pub dns_hijack: bool,
    pub sniffing: Option<SniffingRequest>,
    pub auto_route: bool,
    pub strict_route: bool,
}

fn default_gateway() -> Ipv4Net {
    "172.19.0.1/30".parse().unwrap()
}

impl Default for TunConfig {
    fn default() -> Self {
        Self {
            enable: true,
            name: "tun0".into(),
            device_id: "tun0".into(),
            route_all: false,
            routes: Vec::new(),
            gateway: default_gateway(),
            gateway_v6: None,
            mtu: 1500,
            so_mark: None,
            dns_hijack: false,
            sniffing: None,
            auto_route: false,
            strict_route: false,
        }
    }
}

impl TunConfig {
    pub fn new(name: impl Into<String>, mtu: usize) -> Self {
        let name_str = name.into();
        Self {
            device_id: name_str.clone(),
            name: name_str,
            mtu,
            ..Default::default()
        }
    }
}
