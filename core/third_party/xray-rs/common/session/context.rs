// Module: common\session\context.rs
// 1:1 Rust implementation corresponding to Go common\session\context.go

use super::{Content, Inbound, Outbound};
use crate::common::ctx::Context;
use std::sync::Arc;

pub use crate::common::protocol::SessionContext;

pub fn context_with_inbound(ctx: &Context, inbound: Arc<Inbound>) {
    ctx.set("inbound", inbound);
}

pub fn inbound_from_context(ctx: &Context) -> Option<Arc<Inbound>> {
    ctx.get("inbound")
}

pub fn context_with_outbounds(ctx: &Context, outbounds: Vec<Outbound>) {
    ctx.set("outbounds", outbounds);
}

pub fn outbounds_from_context(ctx: &Context) -> Option<Vec<Outbound>> {
    ctx.get("outbounds")
}

pub fn context_with_content(ctx: &Context, content: Arc<Content>) {
    ctx.set("content", content);
}

pub fn content_from_context(ctx: &Context) -> Option<Arc<Content>> {
    ctx.get("content")
}

pub fn context_with_is_reverse_mux(ctx: &Context, is_reverse: bool) {
    ctx.set("is_reverse_mux", is_reverse);
}

pub fn is_reverse_mux_from_context(ctx: &Context) -> bool {
    ctx.get::<bool>("is_reverse_mux").unwrap_or(false)
}

pub fn get_forced_outbound_tag_from_context(ctx: &Context) -> String {
    ctx.get::<String>("forced_outbound_tag").unwrap_or_default()
}

pub fn set_forced_outbound_tag_to_context(ctx: &Context, tag: impl Into<String>) {
    ctx.set("forced_outbound_tag", tag.into());
}
