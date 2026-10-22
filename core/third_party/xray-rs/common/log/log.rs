// Module: common\log\log.rs
// 1:1 Rust implementation corresponding to Go common\log\log.go

use super::log_pb::Severity;
use std::any::Any;
use std::sync::{Arc, OnceLock, RwLock};

pub type LogLevel = Severity;

/// Message is the trait for all log messages.
pub trait Message: Send + Sync + 'static {
    fn to_log_string(&self) -> String;
    fn severity(&self) -> Option<Severity> {
        None
    }
    fn as_any(&self) -> &dyn Any;
}

/// Handler is the trait for log handler.
pub trait Handler: Send + Sync {
    fn handle(&self, msg: &dyn Message);
}

/// GeneralMessage is a general log message that can contain all kinds of content.
#[derive(Debug, Clone)]
pub struct GeneralMessage {
    pub severity: Severity,
    pub content: String,
}

impl GeneralMessage {
    pub fn new(severity: Severity, content: impl Into<String>) -> Self {
        Self {
            severity,
            content: content.into(),
        }
    }
}

impl Message for GeneralMessage {
    fn to_log_string(&self) -> String {
        format!("[{}] {}", self.severity, self.content)
    }

    fn severity(&self) -> Option<Severity> {
        Some(self.severity)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// syncHandler wraps a Handler with read-write lock synchronization.
pub struct SyncHandler {
    handler: RwLock<Option<Arc<dyn Handler>>>,
}

impl SyncHandler {
    pub fn new() -> Self {
        Self {
            handler: RwLock::new(None),
        }
    }

    pub fn handle(&self, msg: &dyn Message) {
        if let Ok(guard) = self.handler.read()
            && let Some(ref h) = *guard
        {
            h.handle(msg);
        }
    }

    pub fn set(&self, handler: Arc<dyn Handler>) {
        if let Ok(mut guard) = self.handler.write() {
            *guard = Some(handler);
        }
    }
}

impl Default for SyncHandler {
    fn default() -> Self {
        Self::new()
    }
}

static LOG_HANDLER: OnceLock<SyncHandler> = OnceLock::new();

pub fn get_log_handler() -> &'static SyncHandler {
    LOG_HANDLER.get_or_init(SyncHandler::new)
}

/// Record writes a message into log stream.
pub fn record(msg: &dyn Message) {
    get_log_handler().handle(msg);
}

/// RegisterHandler registers a new handler as current log handler.
/// Previous registered handler will be replaced.
pub fn register_handler(handler: Arc<dyn Handler>) {
    get_log_handler().set(handler);
}
