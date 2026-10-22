// Module: common\protocol\http\sniff_test.rs
// 1:1 Rust unit test suite corresponding to Go common\protocol\http\sniff_test.go

#[cfg(test)]
mod tests {
    use super::super::sniff::sniff_http;

    #[test]
    fn test_sniff_http_get() {
        let req =
            b"GET /index.html HTTP/1.1\r\nHost: www.xray.com\r\nUser-Agent: curl/7.68.0\r\n\r\n";
        let header = sniff_http(req).unwrap();
        assert_eq!(header.domain(), "www.xray.com");
        assert_eq!(header.protocol(), "http1");
    }

    #[test]
    fn test_sniff_http_connect() {
        let req = b"CONNECT api.google.com:443 HTTP/1.1\r\nHost: api.google.com:443\r\n\r\n";
        let header = sniff_http(req).unwrap();
        assert_eq!(header.domain(), "api.google.com");
    }
}
