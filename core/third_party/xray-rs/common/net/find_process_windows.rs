// Module: common\net\find_process_windows.rs
// 1:1 Rust implementation corresponding to Go common\net\find_process_windows.go

pub use super::find_process::{ProcessFinder, ProcessInfo};

pub fn find_process_name_by_pid(pid: u32) -> Option<String> {
    if pid == 0 {
        return None;
    }
    #[cfg(target_os = "windows")]
    {
        let info = super::find_process::windows_impl::get_process_info_from_pid(pid);
        Some(info.name)
    }
    #[cfg(not(target_os = "windows"))]
    {
        Some(format!("process_{}", pid))
    }
}
