#[cfg(test)]
mod tests {
    use crate::app::dns::nameserver::{LocalNameServer, NameServer};
    use crate::app::dns::{CachedNameServer, DohNameServer};
    use crate::app::observatory::explain_errors::{ErrorCategory, explain_error};
    use crate::app::proxyman::outbound::uot::{
        UOT_MAGIC_ADDRESS, UotPacket, UotVersion, is_uot_destination,
    };
    use crate::common::net::{Address, Destination};
    use std::sync::Arc;
    use std::time::Duration;

    #[test]
    fn test_uot_packet_encode_decode() {
        let dest = Destination::udp(Address::Domain("1.1.1.1".into()), 53);
        let payload = vec![1, 2, 3, 4, 5];
        let packet = UotPacket::new(dest.clone(), payload.clone());

        // Test Standard (Version 2)
        let encoded_v2 = packet.encode(UotVersion::Standard);
        let (decoded_v2, len_v2) = UotPacket::decode(&encoded_v2, UotVersion::Standard).unwrap();
        assert_eq!(len_v2, encoded_v2.len());
        assert_eq!(decoded_v2.destination.port, 53);
        assert_eq!(decoded_v2.payload, payload);

        // Test Legacy (Version 1)
        let encoded_v1 = packet.encode(UotVersion::Legacy);
        let (decoded_v1, len_v1) = UotPacket::decode(&encoded_v1, UotVersion::Legacy).unwrap();
        assert_eq!(len_v1, encoded_v1.len());
        assert_eq!(decoded_v1.destination.port, 53);
        assert_eq!(decoded_v1.payload, payload);

        let uot_dest = Destination::tcp(Address::Domain(UOT_MAGIC_ADDRESS.into()), 443);
        assert_eq!(is_uot_destination(&uot_dest), Some(UotVersion::Standard));
    }

    #[tokio::test]
    async fn test_cached_nameserver_flow() {
        let local_ns = Arc::new(LocalNameServer::new());
        let cached_ns = CachedNameServer::new(local_ns, Duration::from_secs(60));

        let res1 = cached_ns.query_ip("localhost").await.unwrap();
        assert!(!res1.is_empty());

        let res2 = cached_ns.query_ip("localhost").await.unwrap();
        assert_eq!(res1, res2);
    }

    #[test]
    fn test_doh_nameserver_properties() {
        let doh = DohNameServer::new("https://1.1.1.1/dns-query");
        assert_eq!(doh.url(), "https://1.1.1.1/dns-query");
    }

    #[test]
    fn test_explain_errors_categorization() {
        let (msg1, cat1) = explain_error("i/o timeout on connection");
        assert!(matches!(cat1, ErrorCategory::Timeout));
        assert!(msg1.contains("timeout"));

        let (_msg2, cat2) = explain_error("TLS handshake failed: remote host reset");
        assert!(matches!(cat2, ErrorCategory::TlsHandshakeFailed));

        let (_msg3, cat3) = explain_error("os error 10061: connection refused");
        assert!(matches!(cat3, ErrorCategory::ConnectionRefused));
    }
}
