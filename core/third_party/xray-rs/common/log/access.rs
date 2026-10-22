// Module: common\log\access.rs
// 1:1 Rust implementation corresponding to Go common\log\access.go

use crate::common::ctx::Context;
use crate::common::log::Message;
use crate::common::net::Destination;
use std::fmt;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessStatus {
    Accepted,
    Rejected,
    Redirected,
}

pub const ACCESS_ACCEPTED: &str = "accepted";
pub const ACCESS_REJECTED: &str = "rejected";

impl AccessStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            AccessStatus::Accepted => "accepted",
            AccessStatus::Rejected => "rejected",
            AccessStatus::Redirected => "redirected",
        }
    }
}

impl fmt::Display for AccessStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct AccessMessage {
    pub from: String,
    pub to: String,
    pub status: AccessStatus,
    pub reason: String,
    pub email: String,
    pub detour: String,
}

impl AccessMessage {
    pub fn new(from: impl Into<String>, to: impl Into<String>, status: AccessStatus) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            status,
            reason: String::new(),
            email: String::new(),
            detour: String::new(),
        }
    }
}

impl Message for AccessMessage {
    fn to_log_string(&self) -> String {
        let mut builder = String::new();
        builder.push_str("from ");
        builder.push_str(&self.from);
        builder.push(' ');
        builder.push_str(self.status.as_str());
        builder.push(' ');
        builder.push_str(&self.to);

        if !self.detour.is_empty() {
            builder.push_str(" [");
            builder.push_str(&self.detour);
            builder.push(']');
        }

        if !self.reason.is_empty() {
            builder.push(' ');
            builder.push_str(&self.reason);
        }

        if !self.email.is_empty() {
            builder.push_str(" email: ");
            builder.push_str(&self.email);
        }

        builder
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub fn context_with_access_message(ctx: &Context, msg: Arc<AccessMessage>) {
    ctx.set("access_message", msg);
}

pub fn access_message_from_context(ctx: &Context) -> Option<Arc<AccessMessage>> {
    ctx.get("access_message")
}

/// AccessLogMessage provides compatibility with existing tests and network components.
#[derive(Debug, Clone)]
pub struct AccessLogMessage {
    pub from: Option<SocketAddr>,
    pub to: Destination,
    pub status: AccessStatus,
    pub reason: String,
}

impl AccessLogMessage {
    pub fn format(&self) -> String {
        let from_str = match self.from {
            Some(addr) => addr.to_string(),
            None => "-".to_string(),
        };
        format!(
            "{} {} {} [{}]",
            from_str,
            self.status.as_str(),
            self.to,
            self.reason
        )
    }

    pub fn to_access_message(&self) -> AccessMessage {
        AccessMessage {
            from: match self.from {
                Some(addr) => addr.to_string(),
                None => String::new(),
            },
            to: self.to.to_string(),
            status: self.status,
            reason: self.reason.clone(),
            email: String::new(),
            detour: String::new(),
        }
    }
}
