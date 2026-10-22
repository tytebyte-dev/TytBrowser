use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::sync::{Arc, LazyLock};
use tracing::trace;

pub static DEFAULT_OUTBOUND_INTERFACE: LazyLock<
    Arc<tokio::sync::RwLock<Option<OutboundInterface>>>,
> = LazyLock::new(Default::default);

pub static TUN_SOMARK: LazyLock<tokio::sync::RwLock<Option<u32>>> = LazyLock::new(Default::default);

/// Initialize network configuration
/// globally manage default outbound interface
/// This function should be called as early as possible
/// so that other config initialization can use the default outbound interface
pub async fn init_net_config(tun_somark: Option<u32>) {
    *DEFAULT_OUTBOUND_INTERFACE.write().await = get_outbound_interface();
    *TUN_SOMARK.write().await = tun_somark;

    trace!(
        "default outbound interface: {:?}, tun somark: {:?}",
        *DEFAULT_OUTBOUND_INTERFACE.read().await,
        *TUN_SOMARK.read().await
    );
}

#[derive(Debug, Clone)]
pub struct OutboundInterface {
    pub name: String,
    pub addr_v4: Option<Ipv4Addr>,
    pub netmask_v4: Option<Ipv4Addr>,
    pub broadcast_v4: Option<Ipv4Addr>,
    pub addr_v6: Option<Ipv6Addr>,
    pub netmask_v6: Option<Ipv6Addr>,
    pub broadcast_v6: Option<Ipv6Addr>,
    pub index: u32,
    pub mac_addr: Option<String>,
}

impl From<NetworkInterface> for OutboundInterface {
    fn from(iface: NetworkInterface) -> Self {
        let mut v4 = None;
        let mut v6 = None;

        for addr in iface.addr.iter() {
            if v4.is_some() && v6.is_some() {
                break;
            }
            match addr {
                network_interface::Addr::V4(addr) => {
                    if !addr.ip.is_loopback()
                        && !addr.ip.is_link_local()
                        && !addr.ip.is_unspecified()
                    {
                        v4 = Some(*addr);
                    }
                }
                network_interface::Addr::V6(addr) => {
                    if addr.ip.is_unique_local()
                        || (!addr.ip.is_loopback() && !addr.ip.is_unspecified())
                    {
                        v6 = Some(*addr);
                    }
                }
            }
        }

        OutboundInterface {
            name: iface.name,
            addr_v4: v4.map(|x| x.ip),
            netmask_v4: v4.and_then(|x| x.netmask),
            broadcast_v4: v4.and_then(|x| x.broadcast),
            addr_v6: v6.map(|x| x.ip),
            netmask_v6: v6.and_then(|x| x.netmask),
            broadcast_v6: v6.and_then(|x| x.broadcast),
            index: iface.index,
            mac_addr: iface.mac_addr,
        }
    }
}

pub fn get_interface_by_name(name: &str) -> Option<OutboundInterface> {
    let now = std::time::Instant::now();
    let outbound = network_interface::NetworkInterface::show()
        .ok()?
        .into_iter()
        .find(|iface| iface.name == name)?
        .into();
    trace!(
        "found interface by name: {:?}, took: {}ms",
        outbound,
        now.elapsed().as_millis()
    );
    Some(outbound)
}

pub fn get_outbound_interface() -> Option<OutboundInterface> {
    let now = std::time::Instant::now();

    let mut all_outbounds = network_interface::NetworkInterface::show()
        .ok()?
        .into_iter()
        .map(Into::into)
        .filter(|iface: &OutboundInterface| {
            !iface.name.contains("tun") && (iface.addr_v4.is_some() || iface.addr_v6.is_some())
        })
        .collect::<Vec<_>>();

    let priority: &[&str] = if cfg!(target_os = "windows") {
        &["Ethernet", "Wi-Fi", "Tailscale"]
    } else if cfg!(target_os = "linux") {
        &["eth", "wlp", "en", "Tailscale"]
    } else if cfg!(target_os = "macos") {
        &["en", "pdp_ip", "Tailscale"]
    } else {
        &["eth", "en", "wlp"]
    };

    all_outbounds.sort_by(|left, right| {
        let left_pos = priority
            .iter()
            .position(|x| left.name.contains(x))
            .unwrap_or(usize::MAX);
        let right_pos = priority
            .iter()
            .position(|x| right.name.contains(x))
            .unwrap_or(usize::MAX);
        left_pos.cmp(&right_pos)
    });

    trace!(
        "sorted outbound interfaces: {:?}, took: {}ms",
        all_outbounds,
        now.elapsed().as_millis()
    );

    all_outbounds.into_iter().next()
}
