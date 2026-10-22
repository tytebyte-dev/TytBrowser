// Module: proxy\hysteria\ctx\ctx.rs
// 1:1 Rust implementation corresponding to Go proxy\hysteria\ctx\ctx.go

use crate::common::protocol::SessionContext;

#[derive(Debug, Clone)]
pub struct HysteriaContext {
    pub session: SessionContext,
    pub require_datagram: bool,
}

impl HysteriaContext {
    pub fn new(session: SessionContext) -> Self {
        Self {
            session,
            require_datagram: false,
        }
    }

    pub fn with_require_datagram(mut self, require: bool) -> Self {
        self.require_datagram = require;
        self
    }
}

pub fn context_with_require_datagram(mut ctx: HysteriaContext, udp: bool) -> HysteriaContext {
    ctx.require_datagram = udp;
    ctx
}

pub fn require_datagram_from_context(ctx: &HysteriaContext) -> bool {
    ctx.require_datagram
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::net::Destination;

    #[test]
    fn test_hysteria_context_datagram() {
        let session = SessionContext::new("hysteria-in", Destination::default());
        let ctx = HysteriaContext::new(session);
        assert!(!require_datagram_from_context(&ctx));

        let ctx2 = context_with_require_datagram(ctx, true);
        assert!(require_datagram_from_context(&ctx2));
    }
}
