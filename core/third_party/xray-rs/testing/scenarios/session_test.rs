#[cfg(test)]
mod tests {
    use crate::common::session::{Content, Inbound, Outbound, SniffingRequest, new_session_id};

    #[test]
    fn test_session_id_monotonicity() {
        let id1 = new_session_id();
        let id2 = new_session_id();
        assert!(id2 > id1);
    }

    #[test]
    fn test_session_content_and_attributes() {
        let mut content = Content {
            protocol: "tls".into(),
            sniffing_request: SniffingRequest {
                enabled: true,
                metadata_only: false,
                route_only: true,
                exclude_for_domain: vec!["apple.com".into()],
                override_destination_for_protocol: vec!["tls".into(), "http".into()],
            },
            ..Default::default()
        };

        content.set_attribute("sni", "www.cloudflare.com");
        content.set_attribute("alpn", "h2");

        assert_eq!(content.attribute("sni"), Some("www.cloudflare.com"));
        assert_eq!(content.attribute("alpn"), Some("h2"));
        assert_eq!(content.attribute("nonexistent"), None);
    }

    #[test]
    fn test_inbound_outbound_metadata() {
        let inbound = Inbound {
            tag: "in-10808".into(),
            name: "socks-proxy".into(),
            can_splice_copy: 1,
            ..Default::default()
        };
        assert_eq!(inbound.tag, "in-10808");

        let outbound = Outbound {
            tag: "direct".into(),
            name: "freedom".into(),
            can_splice_copy: 1,
            ..Default::default()
        };
        assert_eq!(outbound.tag, "direct");
    }
}
