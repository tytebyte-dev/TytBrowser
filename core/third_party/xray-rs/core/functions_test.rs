// Module: core\functions_test.rs
// 1:1 Rust unit test suite corresponding to Go core\functions_test.go

#[cfg(test)]
mod tests {
    use crate::common::net::{Address, Destination, Network};
    use crate::core::format::CONFIG_FORMAT_JSON;
    use crate::core::functions::{create_object, dial, dial_udp, start_instance};
    use crate::infra::conf::Config;
    use crate::testing::servers::tcp::{Server as TcpServer, xor_processor};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[test]
    fn test_create_object() {
        let cfg = Config::default();
        let inst = create_object(None, &cfg);
        assert!(inst.is_ok());
    }

    #[test]
    fn test_start_instance_valid_json() {
        let json_cfg = r#"{
            "inbounds": [
                {
                    "protocol": "dokodemo-door",
                    "port": 10888,
                    "settings": {
                        "address": "127.0.0.1",
                        "port": 80
                    }
                }
            ],
            "outbounds": [
                {
                    "protocol": "freedom"
                }
            ]
        }"#;

        let instance = start_instance(CONFIG_FORMAT_JSON, json_cfg.as_bytes());
        assert!(instance.is_ok());
    }

    #[test]
    fn test_start_instance_invalid_json() {
        let invalid = b"{ invalid json }";
        let instance = start_instance(CONFIG_FORMAT_JSON, invalid);
        assert!(instance.is_err());
    }

    #[tokio::test]
    async fn test_xray_dial_stream() {
        let server = TcpServer::start(None, Some(xor_processor(b'c')), None)
            .await
            .unwrap();
        let dest = Destination {
            network: Network::Tcp,
            address: Address::ip(server.addr().ip()),
            port: server.addr().port(),
        };

        let json_cfg = r#"{
            "inbounds": [],
            "outbounds": [
                {
                    "protocol": "freedom"
                }
            ]
        }"#;

        let instance = start_instance(CONFIG_FORMAT_JSON, json_cfg.as_bytes()).unwrap();

        let mut conn = dial(&instance, &dest).await.unwrap();

        let payload = b"Hello Xray Core Dial Test!";
        conn.write_all(payload).await.unwrap();

        let mut buf = vec![0u8; payload.len()];
        conn.read_exact(&mut buf).await.unwrap();

        // Server xors with 'c'
        let expected: Vec<u8> = payload.iter().map(|b| b ^ b'c').collect();
        assert_eq!(buf, expected);

        server.close();
    }

    #[tokio::test]
    async fn test_xray_dial_udp() {
        let json_cfg = r#"{
            "inbounds": [],
            "outbounds": [
                {
                    "protocol": "freedom"
                }
            ]
        }"#;

        let instance = start_instance(CONFIG_FORMAT_JSON, json_cfg.as_bytes()).unwrap();
        let conn = dial_udp(&instance).await;
        assert!(conn.is_ok());
        let udp_conn = conn.unwrap();
        assert!(!udp_conn.is_closed());
        udp_conn.close().await;
        assert!(udp_conn.is_closed());
    }
}
