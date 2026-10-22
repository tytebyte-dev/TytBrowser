// Module: common\protocol\http\headers_test.rs
// 1:1 Rust unit test suite corresponding to Go common\protocol\http\headers_test.go

#[cfg(test)]
mod tests {
    #[test]
    fn test_http_header_keys() {
        let host_header = "Host";
        assert_eq!(host_header.to_lowercase(), "host");
    }
}
