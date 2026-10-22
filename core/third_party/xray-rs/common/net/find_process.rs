// Module: common\net\find_process.rs
// 1:1 Rust implementation corresponding to Go common\net\find_process_windows.go and find_process.go

use super::Destination;
use crate::common::errors::Result;
use std::net::SocketAddr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub path: String,
}

pub struct ProcessFinder;

impl ProcessFinder {
    pub fn find_process(dest: &Destination) -> Result<Option<ProcessInfo>> {
        #[cfg(target_os = "windows")]
        {
            if let Some(addr) = dest.to_socket_addr() {
                let is_tcp = dest.network == crate::common::net::Network::Tcp;
                return Ok(windows_impl::find_process(is_tcp, addr));
            }
            Ok(None)
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = dest;
            Ok(None)
        }
    }

    pub fn find_process_by_socket(src: SocketAddr, is_tcp: bool) -> Result<Option<ProcessInfo>> {
        #[cfg(target_os = "windows")]
        {
            Ok(windows_impl::find_process(is_tcp, src))
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (src, is_tcp);
            Ok(None)
        }
    }
}

#[cfg(target_os = "windows")]
pub mod windows_impl {
    use super::ProcessInfo;
    use std::ffi::c_void;
    use std::net::{IpAddr, SocketAddr};
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::NetworkManagement::IpHelper::{
        GetExtendedTcpTable, GetExtendedUdpTable, TCP_TABLE_OWNER_PID_ALL, UDP_TABLE_OWNER_PID,
    };
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION,
        QueryFullProcessImageNameW,
    };

    const AF_INET: u32 = 2;
    const AF_INET6: u32 = 23;

    pub fn find_process(is_tcp: bool, src: SocketAddr) -> Option<ProcessInfo> {
        let port = src.port();
        let ip = src.ip();
        let family = match ip {
            IpAddr::V4(_) => AF_INET,
            IpAddr::V6(_) => AF_INET6,
        };

        let pid = if is_tcp {
            search_tcp(family, ip, port)?
        } else {
            search_udp(family, ip, port)?
        };

        Some(get_process_info_from_pid(pid))
    }

    fn search_tcp(family: u32, ip: IpAddr, port: u16) -> Option<u32> {
        let mut size: u32 = 0;
        unsafe {
            let _ = GetExtendedTcpTable(None, &mut size, false, family, TCP_TABLE_OWNER_PID_ALL, 0);
        }
        if size == 0 {
            return None;
        }

        let mut buf = vec![0u8; size as usize];
        let ret = unsafe {
            GetExtendedTcpTable(
                Some(buf.as_mut_ptr() as *mut c_void),
                &mut size,
                false,
                family,
                TCP_TABLE_OWNER_PID_ALL,
                0,
            )
        };
        if ret != 0 {
            return None;
        }

        let num_entries = u32::from_ne_bytes(buf[0..4].try_into().ok()?) as usize;
        let (item_size, port_offset, ip_offset, ip_len, pid_offset, _state_offset) =
            if family == AF_INET {
                // MIB_TCPROW_OWNER_PID:
                // dwState (0), dwLocalAddr (4), dwLocalPort (8), dwRemoteAddr (12), dwRemotePort (16), dwOwningPid (20)
                (24, 8, 4, 4, 20, 0)
            } else {
                // MIB_TCP6ROW_OWNER_PID:
                // ucLocalAddr (0), dwLocalScopeId (16), dwLocalPort (20), ucRemoteAddr (24), dwRemoteScopeId (40),
                // dwRemotePort (44), dwState (48), dwOwningPid (52)
                (56, 20, 0, 16, 52, 48)
            };

        for i in 0..num_entries {
            let start = 4 + i * item_size;
            let end = start + item_size;
            if end > buf.len() {
                break;
            }
            let row = &buf[start..end];

            let row_port = u16::from_be_bytes([row[port_offset], row[port_offset + 1]]);
            if row_port != port {
                continue;
            }

            // Check IP match if not unspecified
            let row_ip = &row[ip_offset..ip_offset + ip_len];
            let is_unspecified = row_ip.iter().all(|&b| b == 0);
            if !is_unspecified {
                match ip {
                    IpAddr::V4(v4) => {
                        if row_ip != v4.octets() {
                            continue;
                        }
                    }
                    IpAddr::V6(v6) => {
                        if row_ip != v6.octets() {
                            continue;
                        }
                    }
                }
            }

            let pid = u32::from_ne_bytes(row[pid_offset..pid_offset + 4].try_into().ok()?);
            return Some(pid);
        }

        None
    }

    fn search_udp(family: u32, ip: IpAddr, port: u16) -> Option<u32> {
        let mut size: u32 = 0;
        unsafe {
            let _ = GetExtendedUdpTable(None, &mut size, false, family, UDP_TABLE_OWNER_PID, 0);
        }
        if size == 0 {
            return None;
        }

        let mut buf = vec![0u8; size as usize];
        let ret = unsafe {
            GetExtendedUdpTable(
                Some(buf.as_mut_ptr() as *mut c_void),
                &mut size,
                false,
                family,
                UDP_TABLE_OWNER_PID,
                0,
            )
        };
        if ret != 0 {
            return None;
        }

        let num_entries = u32::from_ne_bytes(buf[0..4].try_into().ok()?) as usize;
        let (item_size, port_offset, ip_offset, ip_len, pid_offset) = if family == AF_INET {
            // MIB_UDPROW_OWNER_PID: dwLocalAddr (0), dwLocalPort (4), dwOwningPid (8)
            (12, 4, 0, 4, 8)
        } else {
            // MIB_UDP6ROW_OWNER_PID: ucLocalAddr (0), dwLocalScopeId (16), dwLocalPort (20), dwOwningPid (24)
            (28, 20, 0, 16, 24)
        };

        for i in 0..num_entries {
            let start = 4 + i * item_size;
            let end = start + item_size;
            if end > buf.len() {
                break;
            }
            let row = &buf[start..end];

            let row_port = u16::from_be_bytes([row[port_offset], row[port_offset + 1]]);
            if row_port != port {
                continue;
            }

            let row_ip = &row[ip_offset..ip_offset + ip_len];
            let is_unspecified = row_ip.iter().all(|&b| b == 0);
            if !is_unspecified {
                match ip {
                    IpAddr::V4(v4) => {
                        if row_ip != v4.octets() {
                            continue;
                        }
                    }
                    IpAddr::V6(v6) => {
                        if row_ip != v6.octets() {
                            continue;
                        }
                    }
                }
            }

            let pid = u32::from_ne_bytes(row[pid_offset..pid_offset + 4].try_into().ok()?);
            return Some(pid);
        }

        None
    }

    pub fn get_process_info_from_pid(pid: u32) -> ProcessInfo {
        if pid == 0 {
            return ProcessInfo {
                pid: 0,
                name: ":System Idle Process".into(),
                path: ":System Idle Process".into(),
            };
        }
        if pid == 4 {
            return ProcessInfo {
                pid: 4,
                name: ":System".into(),
                path: ":System".into(),
            };
        }
        if pid == std::process::id() {
            let cur_exe = std::env::current_exe()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "xray.exe".into());
            let norm_path = cur_exe.replace('\\', "/");
            let name = norm_path
                .split('/')
                .next_back()
                .unwrap_or(&norm_path)
                .trim_end_matches(".exe")
                .to_string();
            return ProcessInfo {
                pid,
                name,
                path: norm_path,
            };
        }

        unsafe {
            if let Ok(handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
                let mut buf = [0u16; 1024];
                let mut len = buf.len() as u32;
                let res = QueryFullProcessImageNameW(
                    handle,
                    PROCESS_NAME_FORMAT(0),
                    windows::core::PWSTR(buf.as_mut_ptr()),
                    &mut len,
                );
                let _ = CloseHandle(handle);
                if res.is_ok() && len > 0 {
                    let path = String::from_utf16_lossy(&buf[..len as usize]);
                    let norm_path = path.replace('\\', "/");
                    let name = norm_path
                        .split('/')
                        .next_back()
                        .unwrap_or(&norm_path)
                        .trim_end_matches(".exe")
                        .to_string();
                    return ProcessInfo {
                        pid,
                        name,
                        path: norm_path,
                    };
                }
            }
        }

        ProcessInfo {
            pid,
            name: format!("pid-{}", pid),
            path: String::new(),
        }
    }
}
