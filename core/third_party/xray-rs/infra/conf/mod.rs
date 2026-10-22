pub mod api;
pub mod blackhole;
pub mod buildable;
pub mod cfgcommon;
pub mod common;
pub mod conf;
pub mod dns;
pub mod dns_proxy;
pub mod dokodemo;
pub mod fakedns;
pub mod freedom;
pub mod grpc;
pub mod http;
pub mod hysteria;
pub mod init;
pub mod json;
pub mod lint;
pub mod loader;
pub mod log;
pub mod loopback;
pub mod metrics;
pub mod observatory;
pub mod policy;
pub mod reverse;
pub mod router;
pub mod router_strategy;
pub mod serial;
pub mod shadowsocks;
pub mod socks;
pub mod transport_authenticators;
pub mod transport_internet;
pub mod trojan;
pub mod tun;
pub mod version;
pub mod vless;
pub mod vmess;
pub mod wireguard;
pub mod xray;

#[cfg(test)]
pub mod api_test;
#[cfg(test)]
pub mod blackhole_test;
#[cfg(test)]
pub mod common_test;
#[cfg(test)]
pub mod dns_proxy_test;
#[cfg(test)]
pub mod dns_test;
#[cfg(test)]
pub mod dokodemo_test;
#[cfg(test)]
pub mod freedom_test;
#[cfg(test)]
pub mod general_test;
#[cfg(test)]
pub mod http_test;
#[cfg(test)]
pub mod loopback_test;
#[cfg(test)]
pub mod metrics_test;
#[cfg(test)]
pub mod policy_test;
#[cfg(test)]
pub mod reverse_test;
#[cfg(test)]
pub mod router_test;
#[cfg(test)]
pub mod shadowsocks_test;
#[cfg(test)]
pub mod socks_test;
#[cfg(test)]
pub mod transport_test;
#[cfg(test)]
pub mod trojan_test;
#[cfg(test)]
pub mod vless_test;
#[cfg(test)]
pub mod vmess_test;
#[cfg(test)]
pub mod wireguard_test;
#[cfg(test)]
pub mod xray_test;

use std::collections::HashMap;

use crate::app::router::{DomainMatcher, IpMatcher, Router, Rule};
use crate::common::errors::{Error, Result};
use crate::common::net::{Destination, Network};
use crate::features::inbound::InboundHandler;
use crate::features::outbound::OutboundHandler;
use crate::proxy as p;
use crate::transport::internet::TlsClient;
use crate::transport::internet::reality::RealityConfig;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub log: Option<LogConfig>,
    #[serde(default)]
    pub dns: Option<serde_json::Value>,
    #[serde(default)]
    pub inbounds: Vec<InboundConfig>,
    #[serde(default)]
    pub outbounds: Vec<OutboundConfig>,
    #[serde(default)]
    pub routing: Option<RoutingConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    #[serde(default)]
    pub loglevel: Option<String>,
    #[serde(default)]
    pub access: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundConfig {
    pub tag: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub listen: Option<String>,
    pub protocol: String,
    #[serde(default)]
    pub settings: Option<serde_json::Value>,
    #[serde(default, rename = "streamSettings")]
    pub stream_settings: Option<StreamSettingsConfig>,
    #[serde(default)]
    pub sniffing: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboundConfig {
    pub tag: Option<String>,
    pub protocol: String,
    #[serde(default)]
    pub settings: Option<serde_json::Value>,
    #[serde(default, rename = "streamSettings")]
    pub stream_settings: Option<StreamSettingsConfig>,
    #[serde(default)]
    pub mux: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamSettingsConfig {
    #[serde(default)]
    pub network: Option<String>,
    #[serde(default)]
    pub security: Option<String>,
    #[serde(default, rename = "tlsSettings")]
    pub tls_settings: Option<TlsSettingsConfig>,
    #[serde(default, rename = "realitySettings")]
    pub reality_settings: Option<RealityConfig>,
    #[serde(default, rename = "wsSettings")]
    pub ws_settings: Option<WsSettingsConfig>,
    #[serde(default)]
    pub finalmask: Option<FinalmaskConfigJson>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalmaskConfigJson {
    #[serde(default)]
    pub tcp: Option<Vec<FinalmaskLayerJson>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalmaskLayerJson {
    #[serde(rename = "type")]
    pub mask_type: String,
    #[serde(default)]
    pub settings: Option<FinalmaskSettingsJson>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalmaskSettingsJson {
    #[serde(default)]
    pub packets: Option<String>,
    #[serde(default)]
    pub length: Option<String>,
    #[serde(default)]
    pub delay: Option<String>,
    #[serde(default, rename = "maxSplit")]
    pub max_split: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsSettingsConfig {
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TlsSettingsConfig {
    #[serde(default, rename = "serverName")]
    pub server_name: Option<String>,
    #[serde(default, rename = "allowInsecure")]
    pub allow_insecure: Option<bool>,
    #[serde(default)]
    pub alpn: Option<Vec<String>>,
    #[serde(default)]
    pub fingerprint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    #[serde(default, rename = "domainStrategy")]
    pub domain_strategy: Option<String>,
    #[serde(default)]
    pub rules: Option<Vec<RuleConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConfig {
    #[serde(default, rename = "type")]
    pub rule_type: Option<String>,
    #[serde(rename = "outboundTag")]
    pub outbound_tag: String,
    #[serde(default, rename = "inboundTag")]
    pub inbound_tag: Option<Vec<String>>,
    #[serde(default)]
    pub domain: Option<Vec<String>>,
    #[serde(default)]
    pub ip: Option<Vec<String>>,
    #[serde(default)]
    pub port: Option<serde_json::Value>,
    #[serde(default)]
    pub network: Option<String>,
    #[serde(default)]
    pub process: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub id: String,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub level: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerTargetConfig {
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub users: Vec<UserConfig>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub method: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiServerSettings {
    #[serde(default)]
    pub vnext: Vec<ServerTargetConfig>,
    #[serde(default)]
    pub servers: Vec<ServerTargetConfig>,
    #[serde(default)]
    pub clients: Vec<UserConfig>,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub network: Option<String>,
}

fn parse_fragment_config(
    fm: &FinalmaskConfigJson,
) -> Option<crate::transport::internet::finalmask::fragment::Config> {
    let tcp_layers = fm.tcp.as_ref()?;
    for layer in tcp_layers {
        if layer.mask_type.eq_ignore_ascii_case("fragment") {
            let mut cfg = crate::transport::internet::finalmask::fragment::Config::default();
            if let Some(ref s) = layer.settings {
                if let Some(ref p) = s.packets {
                    if p.eq_ignore_ascii_case("tlshello") {
                        cfg.packets_from = 0;
                        cfg.packets_to = 1;
                    } else if let Some((from_str, to_str)) = p.split_once('-') {
                        cfg.packets_from = from_str.trim().parse().unwrap_or(0);
                        cfg.packets_to = to_str.trim().parse().unwrap_or(1);
                    }
                }
                if let Some(ref l) = s.length {
                    if let Some((min_str, max_str)) = l.split_once('-') {
                        cfg.length_min = min_str.trim().parse().unwrap_or(50);
                        cfg.length_max = max_str.trim().parse().unwrap_or(100);
                    } else if let Ok(val) = l.trim().parse() {
                        cfg.length_min = val;
                        cfg.length_max = val;
                    }
                }
                if let Some(ref d) = s.delay {
                    if let Some((min_str, max_str)) = d.split_once('-') {
                        cfg.delay_min = min_str.trim().parse().unwrap_or(10);
                        cfg.delay_max = max_str.trim().parse().unwrap_or(20);
                    } else if let Ok(val) = d.trim().parse() {
                        cfg.delay_min = val;
                        cfg.delay_max = val;
                    }
                }
                if let Some(ref ms) = s.max_split {
                    if let Some((min_str, max_str)) = ms.split_once('-') {
                        cfg.max_split_min = min_str.trim().parse().unwrap_or(0);
                        cfg.max_split_max = max_str.trim().parse().unwrap_or(0);
                    } else if let Ok(val) = ms.trim().parse() {
                        cfg.max_split_min = val;
                        cfg.max_split_max = val;
                    }
                }
            }
            return Some(cfg);
        }
    }
    None
}

pub struct BuiltConfig {
    pub inbounds: Vec<(SocketAddr, Arc<dyn InboundHandler>)>,
    pub tun_devices: Vec<Arc<p::tun::WindowsTunDevice>>,
    pub outbounds: HashMap<String, Arc<dyn OutboundHandler>>,
    pub router: Router,
    pub default_outbound_tag: Option<String>,
}

impl Config {
    pub fn build(self) -> Result<BuiltConfig> {
        let mut inbounds = Vec::new();
        let mut tun_devices = Vec::new();
        let mut outbounds = HashMap::new();
        let mut default_outbound_tag = None;
        let dns_client = Arc::new(build_dns_client(self.dns.as_ref()));

        // 1. Build Outbounds
        for (i, out_cfg) in self.outbounds.into_iter().enumerate() {
            let tag = out_cfg.tag.unwrap_or_else(|| format!("outbound-{}", i));
            if default_outbound_tag.is_none() {
                default_outbound_tag = Some(tag.clone());
            }

            let handler: Arc<dyn OutboundHandler> = match out_cfg.protocol.to_lowercase().as_str() {
                "freedom" | "direct" => Arc::new(p::freedom::Handler::new(&tag)),
                "blackhole" | "block" => {
                    let resp_type = out_cfg
                        .settings
                        .as_ref()
                        .and_then(|s| s.get("response"))
                        .and_then(|r| r.get("type"))
                        .and_then(|t| t.as_str())
                        .map(|t| match t.to_lowercase().as_str() {
                            "http" | "http403" => p::blackhole::ResponseType::Http403,
                            "http500" => p::blackhole::ResponseType::Http500,
                            _ => p::blackhole::ResponseType::None,
                        })
                        .unwrap_or(p::blackhole::ResponseType::None);
                    Arc::new(p::blackhole::Handler::with_response(&tag, resp_type))
                }
                "vless" => {
                    let settings: MultiServerSettings =
                        serde_json::from_value(out_cfg.settings.unwrap_or_default()).map_err(
                            |e| Error::Config(format!("Invalid VLESS outbound settings: {}", e)),
                        )?;
                    let server = settings
                        .vnext
                        .first()
                        .ok_or_else(|| Error::Config("VLESS vnext is empty".into()))?;
                    let user = server
                        .users
                        .first()
                        .ok_or_else(|| Error::Config("VLESS users is empty".into()))?;
                    let user_uuid = Uuid::from_str(&user.id)
                        .map_err(|e| Error::Config(format!("Invalid UUID: {}", e)))?;

                    let addr_str = server.address.as_deref().unwrap_or("127.0.0.1");
                    let port_val = server.port.unwrap_or(443);
                    let dest = Destination::from_str(&format!("{}:{}", addr_str, port_val))?;

                    let mut tls_client = None;
                    let mut tls_sni = None;
                    let mut ws_path = None;
                    let mut ws_host = None;
                    let mut fragment_cfg = None;

                    if let Some(stream_settings) = out_cfg.stream_settings {
                        if let Some(ref fm) = stream_settings.finalmask {
                            fragment_cfg = parse_fragment_config(fm);
                        }

                        let is_ws = stream_settings.network.as_deref() == Some("ws")
                            || stream_settings.ws_settings.is_some();
                        if is_ws {
                            let ws_cfg = stream_settings.ws_settings.as_ref();
                            let raw_path = ws_cfg.and_then(|w| w.path.as_deref()).unwrap_or("/");
                            let clean_path = if raw_path.trim().is_empty() {
                                "/"
                            } else {
                                raw_path.trim()
                            };
                            ws_path = Some(clean_path.to_string());

                            if let Some(w) = ws_cfg {
                                if let Some(h) = &w.host
                                    && !h.trim().is_empty()
                                {
                                    ws_host = Some(h.trim().to_string());
                                }
                                if ws_host.is_none()
                                    && let Some(headers) = &w.headers
                                {
                                    for (k, v) in headers {
                                        if k.eq_ignore_ascii_case("host") && !v.trim().is_empty() {
                                            ws_host = Some(v.trim().to_string());
                                            break;
                                        }
                                    }
                                }
                            }
                        }

                        if stream_settings.security.as_deref() == Some("tls") {
                            let tls_cfg = stream_settings.tls_settings.unwrap_or_default();
                            let clean_server_name =
                                tls_cfg.server_name.filter(|s| !s.trim().is_empty());
                            let clean_ws_host = ws_host.clone().filter(|s| !s.trim().is_empty());
                            let sni = clean_server_name
                                .or(clean_ws_host)
                                .unwrap_or_else(|| addr_str.to_string());
                            let allow_insecure = tls_cfg.allow_insecure.unwrap_or(false);
                            let alpn = tls_cfg
                                .alpn
                                .unwrap_or_default()
                                .into_iter()
                                .map(|s| s.into_bytes())
                                .collect();
                            let fp = tls_cfg.fingerprint.as_deref().unwrap_or("");
                            tls_client = Some(TlsClient::new_with_fingerprint(
                                &sni,
                                allow_insecure,
                                alpn,
                                fp,
                            )?);
                            tls_sni = Some(sni.clone());

                            if ws_host.is_none() {
                                ws_host = Some(sni);
                            }
                        }

                        if is_ws && ws_host.is_none() {
                            ws_host = tls_sni.clone().or_else(|| Some(addr_str.to_string()));
                        }
                    }

                    let mut client = p::vless::OutboundClient::new(
                        &tag, dest, user_uuid, tls_client, tls_sni, ws_path, ws_host,
                    );
                    if let Some(f_cfg) = fragment_cfg {
                        client = client.with_fragment(f_cfg);
                    }
                    Arc::new(client)
                }
                "trojan" => {
                    let settings: MultiServerSettings =
                        serde_json::from_value(out_cfg.settings.unwrap_or_default()).map_err(
                            |e| Error::Config(format!("Invalid Trojan outbound settings: {}", e)),
                        )?;
                    let server = settings
                        .servers
                        .first()
                        .ok_or_else(|| Error::Config("Trojan servers list is empty".into()))?;
                    let password = server.password.clone().unwrap_or_default();
                    let addr_str = server.address.as_deref().unwrap_or("127.0.0.1");
                    let port_val = server.port.unwrap_or(443);
                    let dest = Destination::from_str(&format!("{}:{}", addr_str, port_val))?;

                    let mut tls_client = None;
                    let mut tls_sni = None;

                    if let Some(stream_settings) = out_cfg.stream_settings
                        && stream_settings.security.as_deref() == Some("tls")
                    {
                        let tls_cfg = stream_settings.tls_settings.unwrap_or_default();
                        let sni = tls_cfg
                            .server_name
                            .clone()
                            .unwrap_or_else(|| addr_str.to_string());
                        let allow_insecure = tls_cfg.allow_insecure.unwrap_or(false);
                        let alpn = tls_cfg
                            .alpn
                            .unwrap_or_default()
                            .into_iter()
                            .map(|s| s.into_bytes())
                            .collect();
                        let fp = tls_cfg.fingerprint.as_deref().unwrap_or("");
                        tls_client = Some(TlsClient::new_with_fingerprint(
                            &sni,
                            allow_insecure,
                            alpn,
                            fp,
                        )?);
                        tls_sni = Some(sni);
                    }

                    Arc::new(p::trojan::OutboundClient::new(
                        &tag, dest, password, tls_client, tls_sni,
                    ))
                }
                "shadowsocks" => {
                    let settings: MultiServerSettings = serde_json::from_value(
                        out_cfg.settings.unwrap_or_default(),
                    )
                    .map_err(|e| {
                        Error::Config(format!("Invalid Shadowsocks outbound settings: {}", e))
                    })?;
                    let server = settings
                        .servers
                        .first()
                        .ok_or_else(|| Error::Config("Shadowsocks servers list is empty".into()))?;
                    let addr_str = server.address.as_deref().unwrap_or("127.0.0.1");
                    let port_val = server.port.unwrap_or(8388);
                    let dest = Destination::from_str(&format!("{}:{}", addr_str, port_val))?;
                    let password = server.password.clone().unwrap_or_default();
                    let method = server
                        .method
                        .clone()
                        .unwrap_or_else(|| "aes-256-gcm".into());

                    Arc::new(p::shadowsocks::OutboundClient::new(
                        &tag, dest, method, password,
                    ))
                }
                "vmess" => {
                    let settings: MultiServerSettings =
                        serde_json::from_value(out_cfg.settings.unwrap_or_default()).map_err(
                            |e| Error::Config(format!("Invalid VMess outbound settings: {}", e)),
                        )?;
                    let server = settings
                        .vnext
                        .first()
                        .ok_or_else(|| Error::Config("VMess vnext is empty".into()))?;
                    let user = server
                        .users
                        .first()
                        .ok_or_else(|| Error::Config("VMess users is empty".into()))?;
                    let user_uuid = Uuid::from_str(&user.id)
                        .map_err(|e| Error::Config(format!("Invalid UUID: {}", e)))?;
                    let addr_str = server.address.as_deref().unwrap_or("127.0.0.1");
                    let port_val = server.port.unwrap_or(443);
                    let dest = Destination::from_str(&format!("{}:{}", addr_str, port_val))?;

                    let mut tls_client = None;
                    let mut tls_sni = None;

                    if let Some(stream_settings) = out_cfg.stream_settings
                        && stream_settings.security.as_deref() == Some("tls")
                    {
                        let tls_cfg = stream_settings.tls_settings.unwrap_or_default();
                        let sni = tls_cfg
                            .server_name
                            .clone()
                            .unwrap_or_else(|| addr_str.to_string());
                        let allow_insecure = tls_cfg.allow_insecure.unwrap_or(false);
                        let alpn = tls_cfg
                            .alpn
                            .unwrap_or_default()
                            .into_iter()
                            .map(|s| s.into_bytes())
                            .collect();
                        let fp = tls_cfg.fingerprint.as_deref().unwrap_or("");
                        tls_client = Some(TlsClient::new_with_fingerprint(
                            &sni,
                            allow_insecure,
                            alpn,
                            fp,
                        )?);
                        tls_sni = Some(sni);
                    }

                    Arc::new(p::vmess::OutboundClient::new(
                        &tag, dest, user_uuid, tls_client, tls_sni,
                    ))
                }
                "socks" => {
                    let settings: MultiServerSettings =
                        serde_json::from_value(out_cfg.settings.unwrap_or_default()).map_err(
                            |e| Error::Config(format!("Invalid SOCKS outbound settings: {}", e)),
                        )?;
                    let server = settings
                        .servers
                        .first()
                        .ok_or_else(|| Error::Config("SOCKS servers list is empty".into()))?;
                    let addr_str = server.address.as_deref().unwrap_or("127.0.0.1");
                    let port_val = server.port.unwrap_or(1080);
                    let dest = Destination::from_str(&format!("{}:{}", addr_str, port_val))?;

                    Arc::new(p::socks::Client::new(&tag, dest))
                }
                "dns" => Arc::new(p::DnsOutbound::new(tag.clone(), dns_client.clone())),
                other => {
                    return Err(Error::Unsupported(format!(
                        "Outbound protocol '{}' is not supported",
                        other
                    )));
                }
            };

            outbounds.insert(tag, handler);
        }

        // 2. Build Inbounds
        for (i, in_cfg) in self.inbounds.into_iter().enumerate() {
            let tag = in_cfg.tag.unwrap_or_else(|| format!("inbound-{}", i));
            let proto = in_cfg.protocol.to_lowercase();
            if proto == "tun" {
                let tun_name = in_cfg
                    .settings
                    .as_ref()
                    .and_then(|s| s.get("name").and_then(|v| v.as_str()))
                    .unwrap_or("xray_tun");
                let mtu = in_cfg
                    .settings
                    .as_ref()
                    .and_then(|s| {
                        s.get("mtu")
                            .or_else(|| s.get("MTU"))
                            .and_then(|v| v.as_u64())
                    })
                    .unwrap_or(1500) as usize;

                let mut gateway_v4: ipnet::Ipv4Net = "172.18.0.1/30".parse().unwrap();
                let mut gateway_v6: Option<ipnet::Ipv6Net> = None;

                if let Some(gateways) = in_cfg
                    .settings
                    .as_ref()
                    .and_then(|s| s.get("gateway").and_then(|v| v.as_array()))
                {
                    for gw in gateways {
                        if let Some(gw_str) = gw.as_str() {
                            if let Ok(net) = gw_str.parse::<ipnet::Ipv4Net>() {
                                gateway_v4 = net;
                            } else if let Ok(net) = gw_str.parse::<ipnet::Ipv6Net>() {
                                gateway_v6 = Some(net);
                            }
                        }
                    }
                }

                let mut route_all = false;
                let mut routes = Vec::new();
                if let Some(rt_val) = in_cfg
                    .settings
                    .as_ref()
                    .and_then(|s| s.get("autoSystemRoutingTable"))
                {
                    if let Some(arr) = rt_val.as_array() {
                        for item in arr {
                            if let Some(s) = item.as_str() {
                                if s == "0.0.0.0/0" || s == "::/0" {
                                    route_all = true;
                                } else if let Ok(net) = s.parse::<ipnet::IpNet>() {
                                    routes.push(net);
                                }
                            }
                        }
                    } else if rt_val.as_bool() == Some(true) {
                        route_all = true;
                    }
                }

                let sniffing_req = in_cfg.sniffing.map(|s| {
                    let enabled = s.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
                    let route_only = s
                        .get("routeOnly")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    let mut dest_override = Vec::new();
                    if let Some(arr) = s.get("destOverride").and_then(|v| v.as_array()) {
                        for item in arr {
                            if let Some(st) = item.as_str() {
                                dest_override.push(st.to_string());
                            }
                        }
                    }
                    crate::common::session::SniffingRequest {
                        enabled,
                        override_destination_for_protocol: dest_override,
                        route_only,
                        ..Default::default()
                    }
                });

                let tun_cfg = p::tun::TunConfig {
                    enable: true,
                    name: tun_name.to_string(),
                    device_id: tun_name.to_string(),
                    route_all,
                    routes,
                    gateway: gateway_v4,
                    gateway_v6,
                    mtu,
                    so_mark: None,
                    dns_hijack: false,
                    sniffing: sniffing_req,
                    auto_route: route_all,
                    strict_route: true,
                };

                let dev = Arc::new(p::tun::TunRunner::new(tun_cfg));
                tun_devices.push(dev);
                continue;
            }

            let listen_ip = in_cfg.listen.as_deref().unwrap_or("0.0.0.0");
            let port_val = in_cfg.port.unwrap_or(0);

            // Skip TCP binding for port 0
            if port_val == 0 {
                continue;
            }

            let bind_addr: SocketAddr =
                format!("{}:{}", listen_ip, port_val).parse().map_err(|e| {
                    Error::Config(format!(
                        "Invalid listen address '{}:{}': {}",
                        listen_ip, port_val, e
                    ))
                })?;

            let handler: Arc<dyn InboundHandler> = match proto.as_str() {
                "socks" | "socks5" => Arc::new(p::socks::Server::new(&tag)),
                "http" => Arc::new(p::http::Server::new(&tag)),
                "mixed" => Arc::new(p::mixed::Server::new(&tag)),
                "vless" => {
                    let settings: MultiServerSettings = serde_json::from_value(
                        in_cfg.settings.unwrap_or_default(),
                    )
                    .map_err(|e| Error::Config(format!("Invalid VLESS inbound settings: {}", e)))?;
                    let mut user_uuids = Vec::new();
                    for client in settings.clients {
                        let u = Uuid::from_str(&client.id)
                            .map_err(|e| Error::Config(format!("Invalid UUID: {}", e)))?;
                        user_uuids.push(u);
                    }
                    Arc::new(p::vless::InboundServer::new(&tag, user_uuids))
                }
                "trojan" => {
                    let settings: MultiServerSettings =
                        serde_json::from_value(in_cfg.settings.unwrap_or_default()).map_err(
                            |e| Error::Config(format!("Invalid Trojan inbound settings: {}", e)),
                        )?;
                    let mut passwords = Vec::new();
                    for client in settings.clients {
                        if let Some(pwd) = client.password {
                            passwords.push(pwd);
                        }
                    }
                    Arc::new(p::trojan::InboundServer::new(&tag, passwords))
                }
                "shadowsocks" => {
                    let settings: MultiServerSettings = serde_json::from_value(
                        in_cfg.settings.unwrap_or_default(),
                    )
                    .map_err(|e| {
                        Error::Config(format!("Invalid Shadowsocks inbound settings: {}", e))
                    })?;
                    let password = settings
                        .servers
                        .first()
                        .and_then(|s| s.password.clone())
                        .unwrap_or_default();
                    let method = settings
                        .servers
                        .first()
                        .and_then(|s| s.method.clone())
                        .unwrap_or_else(|| "aes-256-gcm".into());
                    Arc::new(p::shadowsocks::InboundServer::new(&tag, method, password))
                }
                "dokodemo-door" | "dokodemo" => {
                    let settings: MultiServerSettings =
                        serde_json::from_value(in_cfg.settings.unwrap_or_default()).map_err(
                            |e| Error::Config(format!("Invalid Dokodemo inbound settings: {}", e)),
                        )?;
                    let target_addr = settings.address.unwrap_or_else(|| "127.0.0.1".into());
                    let target_port = settings.port.unwrap_or(80);
                    let dest = Destination::from_str(&format!("{}:{}", target_addr, target_port))?;
                    let network = match settings.network.as_deref() {
                        Some("udp") => Network::Udp,
                        _ => Network::Tcp,
                    };
                    Arc::new(p::dokodemo::Server::new(&tag, dest, network))
                }
                "vmess" => {
                    let settings: MultiServerSettings = serde_json::from_value(
                        in_cfg.settings.unwrap_or_default(),
                    )
                    .map_err(|e| Error::Config(format!("Invalid VMess inbound settings: {}", e)))?;
                    let mut user_uuids = Vec::new();
                    for client in settings.clients {
                        let u = Uuid::from_str(&client.id)
                            .map_err(|e| Error::Config(format!("Invalid UUID: {}", e)))?;
                        user_uuids.push(u);
                    }
                    Arc::new(p::vmess::InboundServer::new(&tag, user_uuids))
                }
                other => {
                    return Err(Error::Unsupported(format!(
                        "Inbound protocol '{}' is not supported",
                        other
                    )));
                }
            };

            inbounds.push((bind_addr, handler));
        }

        // 3. Build Router
        let mut rules = Vec::new();
        if let Some(routing) = self.routing
            && let Some(rule_configs) = routing.rules
        {
            for r_cfg in rule_configs {
                let mut rule = Rule::new(&r_cfg.outbound_tag);
                if let Some(in_tags) = r_cfg.inbound_tag {
                    rule.inbound_tags = in_tags;
                }
                if let Some(domains) = r_cfg.domain {
                    for d in domains {
                        rule.domain_matchers.push(DomainMatcher::parse(&d));
                    }
                }
                if let Some(ips) = r_cfg.ip {
                    for ip_str in ips {
                        if let Some(ip_matcher) = IpMatcher::parse(&ip_str) {
                            rule.ip_matchers.push(ip_matcher);
                        }
                    }
                }
                if let Some(port_val) = r_cfg.port {
                    let ports_str = match port_val {
                        serde_json::Value::Number(n) => n.to_string(),
                        serde_json::Value::String(s) => s,
                        _ => String::new(),
                    };
                    for part in ports_str.split(',') {
                        let trimmed = part.trim();
                        if trimmed.contains('-') {
                            let range_parts: Vec<&str> = trimmed.split('-').collect();
                            if range_parts.len() == 2
                                && let (Ok(start), Ok(end)) = (
                                    range_parts[0].trim().parse::<u16>(),
                                    range_parts[1].trim().parse::<u16>(),
                                )
                            {
                                for p in start..=end {
                                    rule.ports.push(p);
                                }
                            }
                        } else if let Ok(p) = trimmed.parse::<u16>() {
                            rule.ports.push(p);
                        }
                    }
                }
                if let Some(net_str) = r_cfg.network {
                    rule.network = match net_str.to_lowercase().as_str() {
                        "tcp" => Some(Network::Tcp),
                        "udp" => Some(Network::Udp),
                        _ => None,
                    };
                }
                if let Some(proc_list) = r_cfg.process {
                    rule.process = proc_list;
                }
                rules.push(rule);
            }
        }

        let router = Router::new(rules, default_outbound_tag.clone());

        Ok(BuiltConfig {
            inbounds,
            tun_devices,
            outbounds,
            router,
            default_outbound_tag,
        })
    }
}

fn build_dns_client(dns_val: Option<&serde_json::Value>) -> crate::app::dns::DnsClient {
    let mut client = crate::app::dns::DnsClient::new();
    if let Some(val) = dns_val {
        if let Some(hosts_obj) = val.get("hosts").and_then(|h| h.as_object()) {
            for (domain, ip_val) in hosts_obj {
                let mut ips = Vec::new();
                if let Some(s) = ip_val.as_str() {
                    if let Ok(ip) = s.parse::<std::net::IpAddr>() {
                        ips.push(ip);
                    }
                } else if let Some(arr) = ip_val.as_array() {
                    for item in arr {
                        if let Some(s) = item.as_str()
                            && let Ok(ip) = s.parse::<std::net::IpAddr>()
                        {
                            ips.push(ip);
                        }
                    }
                }
                if !ips.is_empty() {
                    client.add_host(domain, ips);
                }
            }
        }

        if let Some(servers_arr) = val.get("servers").and_then(|s| s.as_array()) {
            for s in servers_arr {
                let addr_str = if let Some(addr) = s.as_str() {
                    Some(addr.to_string())
                } else if let Some(obj) = s.as_object() {
                    obj.get("address")
                        .and_then(|v| v.as_str())
                        .map(|v| v.to_string())
                } else {
                    None
                };
                if let Some(addr) = addr_str {
                    client.add_server(crate::app::dns::DnsServerConfig {
                        address: addr,
                        domains: Vec::new(),
                        skip_fallback: false,
                        tag: None,
                    });
                }
            }
        }
    }
    client
}
