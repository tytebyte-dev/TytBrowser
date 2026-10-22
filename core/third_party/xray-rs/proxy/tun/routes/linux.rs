use super::super::config::TunConfig;
use super::super::net::OutboundInterface;
use ipnet::IpNet;
use tracing::warn;

pub fn check_ip_command_installed() -> std::io::Result<()> {
    std::process::Command::new("ip")
        .arg("route")
        .output()
        .and_then(|output| {
            if output.status.success() {
                Ok(())
            } else {
                Err(std::io::Error::other("ip command not found"))
            }
        })
}

pub fn add_route(via: &OutboundInterface, dest: &IpNet) -> std::io::Result<()> {
    let cmd = std::process::Command::new("ip")
        .arg("route")
        .arg("add")
        .arg(dest.to_string())
        .arg("dev")
        .arg(&via.name)
        .output()?;
    warn!("executing: ip route add {} dev {}", dest, via.name);
    if !cmd.status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("add route failed: {}", String::from_utf8_lossy(&cmd.stderr)),
        ));
    }
    Ok(())
}

pub fn setup_policy_routing(_cfg: &TunConfig, via: &OutboundInterface) -> std::io::Result<()> {
    warn!("setting up policy routing on Linux for {}", via.name);
    let _ = std::process::Command::new("ip")
        .args(["route", "add", "default", "dev", &via.name])
        .output();
    Ok(())
}

pub fn maybe_routes_clean_up(_cfg: &TunConfig, _tun_name: &str) -> std::io::Result<()> {
    Ok(())
}
