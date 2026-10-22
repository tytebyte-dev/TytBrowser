// Module: common\session\session_test.rs
// 1:1 Rust unit test suite corresponding to Go common\session\session_test.go

#[cfg(test)]
mod tests {
    use crate::common::ctx::Context;
    use crate::common::net::{Address, Destination};
    use crate::common::session::context::{
        content_from_context, context_with_content, context_with_inbound,
        context_with_is_reverse_mux, context_with_outbounds, get_forced_outbound_tag_from_context,
        inbound_from_context, is_reverse_mux_from_context, outbounds_from_context,
        set_forced_outbound_tag_to_context,
    };
    use crate::common::session::{Content, Inbound, Outbound};
    use std::sync::Arc;

    #[test]
    fn test_session_inbound_and_outbound_context() {
        let ctx = Context::new();

        let mut inbound = Inbound::default();
        inbound.tag = "socks-in".to_string();
        inbound.source = Some(Destination::tcp(Address::Domain("127.0.0.1".into()), 1080));
        context_with_inbound(&ctx, Arc::new(inbound));

        let retrieved_inbound = inbound_from_context(&ctx).expect("retrieve inbound");
        assert_eq!(retrieved_inbound.tag, "socks-in");

        let mut outbounds = Vec::new();
        let mut ob = Outbound::default();
        ob.tag = "direct".to_string();
        ob.target = Some(Destination::tcp(Address::Domain("example.com".into()), 443));
        outbounds.push(ob);
        context_with_outbounds(&ctx, outbounds);

        let retrieved_outbounds = outbounds_from_context(&ctx).expect("retrieve outbounds");
        assert_eq!(retrieved_outbounds.len(), 1);
        assert_eq!(retrieved_outbounds[0].tag, "direct");

        let mut content = Content::default();
        content.protocol = "http".to_string();
        content.set_attribute("key", "val");
        context_with_content(&ctx, Arc::new(content));

        let retrieved_content = content_from_context(&ctx).expect("retrieve content");
        assert_eq!(retrieved_content.protocol, "http");
        assert_eq!(retrieved_content.attribute("key"), Some("val"));

        assert!(!is_reverse_mux_from_context(&ctx));
        context_with_is_reverse_mux(&ctx, true);
        assert!(is_reverse_mux_from_context(&ctx));

        set_forced_outbound_tag_to_context(&ctx, "proxy-out");
        assert_eq!(get_forced_outbound_tag_from_context(&ctx), "proxy-out");
    }
}
