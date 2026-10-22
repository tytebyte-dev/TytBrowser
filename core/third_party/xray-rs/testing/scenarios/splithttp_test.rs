#[cfg(test)]
mod tests {
    use crate::transport::internet::splithttp::{SplitHttpClient, SplitHttpConfig};

    #[test]
    fn test_splithttp_request_formatting() {
        let mut cfg = SplitHttpConfig::new("/xhttp-stream", "custom.domain.com");
        cfg.headers
            .insert("User-Agent".into(), "Mozilla/5.0".into());

        let client = SplitHttpClient::new(cfg, "session-uuid-12345");

        let download_req = client.format_download_request();
        assert!(download_req.starts_with("GET /xhttp-stream HTTP/1.1\r\n"));
        assert!(download_req.contains("Host: custom.domain.com\r\n"));
        assert!(download_req.contains("X-Session-Id: session-uuid-12345\r\n"));
        assert!(download_req.contains("User-Agent: Mozilla/5.0\r\n"));
        assert!(download_req.ends_with("\r\n\r\n"));

        let upload_req = client.format_upload_request(4096);
        assert!(upload_req.starts_with("POST /xhttp-stream HTTP/1.1\r\n"));
        assert!(upload_req.contains("Content-Length: 4096\r\n"));
        assert!(upload_req.contains("X-Session-Id: session-uuid-12345\r\n"));
    }
}
