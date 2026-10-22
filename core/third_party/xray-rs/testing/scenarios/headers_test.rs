#[cfg(test)]
mod tests {
    use crate::transport::internet::finalmask::header::dns::DnsHeader;
    use crate::transport::internet::finalmask::header::dtls::DtlsHeader;
    use crate::transport::internet::finalmask::header::srtp::SrtpHeader;
    use crate::transport::internet::finalmask::header::utp::UtpHeader;
    use crate::transport::internet::finalmask::header::wechat::WeChatHeader;
    use crate::transport::internet::finalmask::header::wireguard::WireguardHeader;

    #[test]
    fn test_srtp_header_serialization() {
        let mut srtp = SrtpHeader::new();
        assert_eq!(srtp.size(), 4);
        let h1 = srtp.write_header();
        assert_eq!(&h1[0..2], &[0xB5, 0xE8]);
        let h2 = srtp.write_header();
        assert_eq!(&h2[0..2], &[0xB5, 0xE8]);
        let num1 = u16::from_be_bytes([h1[2], h1[3]]);
        let num2 = u16::from_be_bytes([h2[2], h2[3]]);
        assert_eq!(num2, num1.wrapping_add(1));
    }

    #[test]
    fn test_wechat_header_serialization() {
        let mut wechat = WeChatHeader::new();
        assert_eq!(wechat.size(), 13);
        let h = wechat.write_header();
        assert_eq!(h[0], 0xa1);
        assert_eq!(h[1], 0x08);
        assert_eq!(&h[6..13], &[0x00, 0x10, 0x11, 0x18, 0x30, 0x22, 0x30]);
    }

    #[test]
    fn test_utp_header_serialization() {
        let utp = UtpHeader::new();
        assert_eq!(utp.size(), 4);
        let h = utp.write_header();
        assert_eq!(h[2], 1); // type = ST_DATA
        assert_eq!(h[3], 0); // extension = 0
    }

    #[test]
    fn test_wireguard_header_serialization() {
        let wg = WireguardHeader::new();
        assert_eq!(wg.size(), 4);
        assert_eq!(wg.write_header(), [0x04, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_dtls_header_serialization() {
        let mut dtls = DtlsHeader::new();
        assert_eq!(dtls.size(), 13);
        let h = dtls.write_header();
        assert_eq!(h[0], 23); // Application Data
        assert_eq!(h[1], 254); // DTLS 1.2
        assert_eq!(h[2], 253);
    }

    #[test]
    fn test_dns_header_pack() {
        let mut buf = [0u8; 128];
        let len = DnsHeader::pack_domain_name("www.google.com", &mut buf).unwrap();
        // 3 + "www" (3) + 6 + "google" (6) + 3 + "com" (3) + 1 (null) = 16 bytes
        assert_eq!(len, 16);
        assert_eq!(buf[0], 3);
        assert_eq!(&buf[1..4], b"www");
        assert_eq!(buf[4], 6);
        assert_eq!(&buf[5..11], b"google");
        assert_eq!(buf[11], 3);
        assert_eq!(&buf[12..15], b"com");
        assert_eq!(buf[15], 0);

        let dns = DnsHeader::new("cloudflare.com");
        let mut pkt = [0u8; 128];
        let pkt_len = dns.serialize(&mut pkt).unwrap();
        assert!(pkt_len > 12);
        assert_eq!(&pkt[2..4], &[0x01, 0x00]); // Standard query
    }

    #[tokio::test]
    async fn test_http_header_obfuscator() {
        use crate::transport::internet::headers::http::{
            HttpHeaderObfuscator, RequestConfig, ResponseConfig,
        };
        use std::io::Cursor;

        let req_cfg = RequestConfig::new();
        let formatted = req_cfg.format_request("my-site.com");
        assert!(formatted.starts_with("GET / HTTP/1.1\r\nHost: my-site.com\r\n"));
        assert!(formatted.ends_with("\r\n\r\n"));

        let resp_cfg = ResponseConfig::new();
        let resp_formatted = resp_cfg.format_response();
        assert!(resp_formatted.starts_with("HTTP/1.1 200 OK\r\n"));

        let mut stream_data = formatted.into_bytes();
        stream_data.extend_from_slice(b"payload-data-after-http-header");

        let mut cursor = Cursor::new(stream_data);
        let (header, trailing) = HttpHeaderObfuscator::server_read_request(&mut cursor)
            .await
            .unwrap();
        assert!(header.contains("Host: my-site.com"));
        assert_eq!(trailing, b"payload-data-after-http-header");
    }
}
