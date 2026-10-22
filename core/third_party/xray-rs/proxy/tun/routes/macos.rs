use super::super::config::TunConfig;
use super::super::net::OutboundInterface;
use ipnet::IpNet;
use tracing::warn;

pub fn add_route(via: &OutboundInterface, dest: &IpNet) -> std::io::Result<()> {
    let mut cmd = std::process::Command::new("route");
    cmd.arg("add");
    match dest {
        IpNet::V4(_) => {
            cmd.arg("-net")
                .arg(dest.to_string())
                .arg("-interface")
                .arg(&via.name);
            warn!("executing: route add -net {} -interface {}", dest, via.name);
        }
        IpNet::V6(_) => {
            cmd.arg("-inet6")
                .arg(dest.to_string())
                .arg("-interface")
                .arg(&via.name);
            warn!(
                "executing: route add -inet6 {} -interface {}",
                dest, via.name
            );
        }
    }
    let output = cmd.output()?;
    if !output.status.success() {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "add route failed",
        ))
    } else {
        Ok(())
    }
}

pub fn maybe_add_default_route() -> std::io::Result<()> {
    Ok(())
}

pub fn maybe_routes_clean_up(_cfg: &TunConfig, _tun_name: &str) -> std::io::Result<()> {
    Ok(())
}
