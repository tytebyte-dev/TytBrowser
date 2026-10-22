// Module: common\singbridge\logger.rs
// 1:1 Rust implementation corresponding to Go common\singbridge\logger.go

use crate::common::ctx::Context;
use crate::common::log::{GeneralMessage, Severity, record};

/// ContextLogger trait 1:1 corresponding to sagernet/sing/common/logger.ContextLogger.
pub trait ContextLogger: Send + Sync {
    fn trace(&self, msg: &str);
    fn debug(&self, msg: &str);
    fn info(&self, msg: &str);
    fn warn(&self, msg: &str);
    fn error(&self, msg: &str);
    fn fatal(&self, msg: &str);
    fn panic(&self, msg: &str);

    fn trace_context(&self, ctx: &Context, msg: &str);
    fn debug_context(&self, ctx: &Context, msg: &str);
    fn info_context(&self, ctx: &Context, msg: &str);
    fn warn_context(&self, ctx: &Context, msg: &str);
    fn error_context(&self, ctx: &Context, msg: &str);
    fn fatal_context(&self, ctx: &Context, msg: &str);
    fn panic_context(&self, ctx: &Context, msg: &str);
}

/// XrayLogger forwards sing logger calls into Xray logging infrastructure.
/// 1:1 corresponding to XrayLogger in logger.go.
pub struct XrayLogger {
    prefix: String,
}

impl XrayLogger {
    pub fn new() -> Self {
        Self {
            prefix: "[sing-bridge]".to_string(),
        }
    }

    pub fn with_prefix(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
        }
    }
}

impl Default for XrayLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextLogger for XrayLogger {
    fn trace(&self, _msg: &str) {}

    fn debug(&self, msg: &str) {
        record(&GeneralMessage {
            severity: Severity::Debug,
            content: format!("{} {}", self.prefix, msg),
        });
    }

    fn info(&self, msg: &str) {
        record(&GeneralMessage {
            severity: Severity::Info,
            content: format!("{} {}", self.prefix, msg),
        });
    }

    fn warn(&self, msg: &str) {
        record(&GeneralMessage {
            severity: Severity::Warning,
            content: format!("{} {}", self.prefix, msg),
        });
    }

    fn error(&self, msg: &str) {
        record(&GeneralMessage {
            severity: Severity::Error,
            content: format!("{} {}", self.prefix, msg),
        });
    }

    fn fatal(&self, _msg: &str) {}
    fn panic(&self, _msg: &str) {}

    fn trace_context(&self, _ctx: &Context, _msg: &str) {}
    fn debug_context(&self, _ctx: &Context, msg: &str) {
        self.debug(msg);
    }
    fn info_context(&self, _ctx: &Context, msg: &str) {
        self.info(msg);
    }
    fn warn_context(&self, _ctx: &Context, msg: &str) {
        self.warn(msg);
    }
    fn error_context(&self, _ctx: &Context, msg: &str) {
        self.error(msg);
    }
    fn fatal_context(&self, _ctx: &Context, _msg: &str) {}
    fn panic_context(&self, _ctx: &Context, _msg: &str) {}
}

/// Backwards compatibility alias
pub type SingLogger = XrayLogger;
