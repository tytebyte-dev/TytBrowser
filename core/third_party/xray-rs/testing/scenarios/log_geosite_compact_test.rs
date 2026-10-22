#[cfg(test)]
mod tests {
    use crate::app::commander::{Commander, CommanderOutbound};
    use crate::app::log::{LogCreatorOptions, LogLevel, create_logger};
    use crate::app::router::GeoSiteCompactList;
    use crate::common::net::{Address, Destination};
    use crate::common::protocol::SessionContext;
    use crate::features::outbound::OutboundHandler;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_commander_outbound_lifecycle() {
        let commander = Arc::new(Commander::new(
            "commander-in".into(),
            "127.0.0.1:10085".into(),
        ));
        let outbound = CommanderOutbound::new("commander-out", commander.clone());
        assert_eq!(outbound.tag(), "commander-out");
        assert_eq!(outbound.commander().tag, "commander-in");

        let dest = Destination::tcp(Address::Domain("localhost".into()), 10085);
        let session = SessionContext::new("commander-in", dest);
        let stream = outbound.connect(&session).await;
        assert!(stream.is_ok());
    }

    #[test]
    fn test_log_creator_options_and_creation() {
        let console_opt = LogCreatorOptions::console(LogLevel::Debug);
        assert!(create_logger(&console_opt).is_ok());

        let none_opt = LogCreatorOptions::none();
        assert!(create_logger(&none_opt).is_ok());
    }

    #[test]
    fn test_geosite_compact_list_with_deps() {
        let mut list = GeoSiteCompactList::new();
        list.add_site("cn", vec!["baidu.com".into(), "qq.com".into()]);
        list.add_site("google", vec!["google.com".into(), "youtube.com".into()]);
        list.add_dep("cn", "google");

        let all = list.get_all_domains("cn");
        assert_eq!(all.len(), 4);
        assert!(all.contains(&"baidu.com".to_string()));
        assert!(all.contains(&"youtube.com".to_string()));
    }
}
