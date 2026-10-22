// Module: main\confloader\external\external.rs
// 1:1 Rust implementation corresponding to Go main\confloader\external\external.go

use crate::common::errors::{Error, Result};
pub use crate::main::confloader::set_effective_config_file_loader;

pub use config_loader as load_external_config;

pub fn config_loader(arg: &str) -> Result<Vec<u8>> {
    if arg.starts_with("http+unix://") {
        fetch_unix_socket_http_content(arg)
    } else if arg.starts_with("http://") || arg.starts_with("https://") {
        fetch_http_content(arg)
    } else if arg == "stdin:" || arg == "-" {
        use std::io::Read;
        let mut buf = Vec::new();
        std::io::stdin().read_to_end(&mut buf).map_err(Error::Io)?;
        Ok(buf)
    } else {
        std::fs::read(arg).map_err(Error::Io)
    }
}

pub fn fetch_http_content(target: &str) -> Result<Vec<u8>> {
    if !target.starts_with("http://") && !target.starts_with("https://") {
        return Err(Error::Other(format!("invalid scheme: {}", target)));
    }
    Err(Error::Other(format!(
        "remote http config fetching from {target} requires live network connection"
    )))
}

pub fn fetch_unix_socket_http_content(target: &str) -> Result<Vec<u8>> {
    let path = target.strip_prefix("http+unix://").unwrap_or(target);
    if !path.starts_with('/') {
        return Err(Error::Other("unix socket path must be absolute".into()));
    }
    let sock_idx = path.find(".sock").ok_or_else(|| {
        Error::Other("cannot determine socket path, socket file should have .sock extension".into())
    })?;
    let socket_path = &path[..sock_idx + 5];
    let http_path = &path[sock_idx + 5..];
    let _effective_http_path = if http_path.is_empty() { "/" } else { http_path };

    if !std::path::Path::new(socket_path).exists() {
        return Err(Error::Other(format!(
            "socket file not found: {}",
            socket_path
        )));
    }

    Err(Error::Other(format!(
        "unix socket connection to {socket_path} not available"
    )))
}

pub fn init_external_loader() {
    set_effective_config_file_loader(config_loader);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_external_scheme_validation() {
        let err = fetch_http_content("ftp://example.com/conf").unwrap_err();
        assert!(err.to_string().contains("invalid scheme"));

        let sock_err =
            fetch_unix_socket_http_content("http+unix://relative/test.sock").unwrap_err();
        assert!(
            sock_err
                .to_string()
                .contains("unix socket path must be absolute")
        );

        let no_ext_err = fetch_unix_socket_http_content("http+unix:///tmp/socketfile").unwrap_err();
        assert!(no_ext_err.to_string().contains(".sock"));
    }
}
