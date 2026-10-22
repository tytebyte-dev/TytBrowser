use super::config::TunConfig;
use super::runner::TunRunner;
use crate::app::dispatcher::DefaultDispatcher;
use crate::common::errors::{Error, Result};
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use tokio::task::JoinHandle;

pub struct TunHandler {
    pub config: TunConfig,
    pub dispatcher: Arc<DefaultDispatcher>,
}

impl TunHandler {
    pub fn new(config: TunConfig, dispatcher: Arc<DefaultDispatcher>) -> Self {
        Self { config, dispatcher }
    }

    pub fn parse_ip_packet(packet: &[u8]) -> Result<(IpAddr, IpAddr, u8, &'static str, u16, u16)> {
        if packet.len() < 20 {
            return Err(Error::Protocol("Packet too short for IPv4 header".into()));
        }

        let version = packet[0] >> 4;
        if version == 4 {
            let ihl = ((packet[0] & 0x0f) * 4) as usize;
            if packet.len() < ihl {
                return Err(Error::Protocol("Invalid IPv4 IHL".into()));
            }

            let proto = packet[9];
            let src = IpAddr::V4(Ipv4Addr::new(
                packet[12], packet[13], packet[14], packet[15],
            ));
            let dst = IpAddr::V4(Ipv4Addr::new(
                packet[16], packet[17], packet[18], packet[19],
            ));

            let (src_port, dst_port, net_type) = if proto == 6 && packet.len() >= ihl + 4 {
                let sp = u16::from_be_bytes([packet[ihl], packet[ihl + 1]]);
                let dp = u16::from_be_bytes([packet[ihl + 2], packet[ihl + 3]]);
                (sp, dp, "tcp")
            } else if proto == 17 && packet.len() >= ihl + 4 {
                let sp = u16::from_be_bytes([packet[ihl], packet[ihl + 1]]);
                let dp = u16::from_be_bytes([packet[ihl + 2], packet[ihl + 3]]);
                (sp, dp, "udp")
            } else {
                (0, 0, "other")
            };

            Ok((src, dst, proto, net_type, src_port, dst_port))
        } else {
            Err(Error::Protocol("IPv6 TUN parsing fallback".into()))
        }
    }

    pub async fn start_device(&self, device: Arc<TunRunner>) -> Result<JoinHandle<()>> {
        device.start(self.dispatcher.clone()).await
    }
}
