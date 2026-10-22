// Module: transport\internet\headers\http\http_test.rs
// 1:1 Rust unit test suite corresponding to Go transport\internet\headers\http\http_test.go

#[cfg(test)]
mod tests {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use super::super::config_pb::{
        Config, Header, Method, RequestConfig, ResponseConfig, Status, Version,
    };
    use super::super::http::{ENDING, HeaderReader, HeaderWriter, new_authenticator};
    use super::super::resp::{resp400, resp404};

    #[tokio::test]
    async fn test_reader_writer() {
        let mut data = Vec::new();
        data.extend_from_slice(b"abcd");
        data.extend_from_slice(ENDING.as_bytes());

        let mut writer = HeaderWriter::new(data);
        let mut target = Vec::new();
        writer.write_to_vec(&mut target);
        assert_eq!(target.len(), 8);

        let reader = HeaderReader::new(vec!["/".into()]);
        let mut valid_http = Vec::new();
        valid_http.extend_from_slice(b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n");
        assert!(reader.parse_and_validate(&valid_http).is_ok());

        let mut mismatch_http = Vec::new();
        mismatch_http.extend_from_slice(b"GET /other HTTP/1.1\r\nHost: example.com\r\n\r\n");
        assert!(reader.parse_and_validate(&mismatch_http).is_err());
    }

    #[tokio::test]
    async fn test_request_header() {
        let auth = new_authenticator(Config {
            request: Some(RequestConfig {
                version: Some(Version {
                    value: "1.1".into(),
                }),
                method: Some(Method {
                    value: "GET".into(),
                }),
                uri: vec!["/".into()],
                header: vec![Header {
                    name: "Test".into(),
                    value: vec!["Value".into()],
                }],
            }),
            response: None,
        });

        let req_bytes = auth.format_request();
        let req_str = String::from_utf8(req_bytes).unwrap();
        assert_eq!(req_str, "GET / HTTP/1.1\r\nTest: Value\r\n\r\n");
    }

    #[tokio::test]
    async fn test_resp400_and_resp404() {
        let r400 = resp400();
        assert_eq!(r400.get_status_code(), "400");
        assert_eq!(r400.get_status_reason(), "Bad Request");
        assert!(r400.has_header("Connection"));
        assert!(r400.has_header("Content-Length"));

        let r404 = resp404();
        assert_eq!(r404.get_status_code(), "404");
        assert_eq!(r404.get_status_reason(), "Not Found");
        assert!(r404.has_header("Connection"));
    }

    #[tokio::test]
    async fn test_connection_duplex() {
        let config = Config {
            request: Some(RequestConfig {
                version: Some(Version {
                    value: "1.1".into(),
                }),
                method: Some(Method {
                    value: "POST".into(),
                }),
                uri: vec!["/testpath".into()],
                header: vec![Header {
                    name: "Host".into(),
                    value: vec!["www.example.com".into()],
                }],
            }),
            response: Some(ResponseConfig {
                version: Some(Version {
                    value: "1.1".into(),
                }),
                status: Some(Status {
                    code: "200".into(),
                    reason: "OK".into(),
                }),
                header: vec![Header {
                    name: "Content-Type".into(),
                    value: vec!["application/octet-stream".into()],
                }],
            }),
        };

        let auth = new_authenticator(config);

        let (client_raw, server_raw) = tokio::io::duplex(65536);
        let mut client_stream = auth.client_stream(client_raw);
        let mut server_stream = auth.server_stream(server_raw);

        // Client writes payload; should automatically prepend HTTP POST request header
        let client_task = tokio::spawn(async move {
            client_stream
                .write_all(b"payload from client")
                .await
                .unwrap();
            let mut buf = vec![0u8; 100];
            let n = client_stream.read(&mut buf).await.unwrap();
            assert_eq!(&buf[..n], b"response from server");
        });

        // Server reads payload; should automatically strip request header and return payload
        let server_task = tokio::spawn(async move {
            let mut buf = vec![0u8; 100];
            let n = server_stream.read(&mut buf).await.unwrap();
            assert_eq!(&buf[..n], b"payload from client");
            // Server writes response; should automatically prepend HTTP 200 response header
            server_stream
                .write_all(b"response from server")
                .await
                .unwrap();
        });

        let (res1, res2) = tokio::join!(client_task, server_task);
        res1.unwrap();
        res2.unwrap();
    }
}
