// Module: transport\internet\internet.rs
// 1:1 Rust implementation corresponding to Go transport\internet\internet.go

pub const DEFAULT_STREAM_BUFFER_SIZE: usize = 64 * 1024;

pub fn is_valid_http_host(request: &str, config: &str) -> bool {
    let r = request.to_lowercase();
    let c = config.to_lowercase();
    if let Some((h, _)) = r.split_once(':') {
        h == c
    } else {
        r == c
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_http_host() {
        assert!(is_valid_http_host("example.com", "example.com"));
        assert!(is_valid_http_host("EXAMPLE.COM", "example.com"));
        assert!(is_valid_http_host("example.com:8080", "example.com"));
        assert!(!is_valid_http_host("other.com", "example.com"));
        assert!(!is_valid_http_host("other.com:443", "example.com"));
    }
}
