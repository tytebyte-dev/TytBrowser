// Module: common\ctx\context.rs
// 1:1 Rust implementation corresponding to Go common\ctx\context.go

use std::any::Any;
use std::collections::HashMap;
use std::sync::RwLock;

pub type ID = u32;

#[derive(Default)]
pub struct Context {
    values: RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>,
}

impl Context {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set<T: 'static + Send + Sync>(&self, key: impl Into<String>, val: T) {
        let mut guard = self.values.write().unwrap();
        guard.insert(key.into(), Box::new(val));
    }

    pub fn get<T: 'static + Clone + Send + Sync>(&self, key: &str) -> Option<T> {
        let guard = self.values.read().unwrap();
        guard.get(key)?.downcast_ref::<T>().cloned()
    }

    pub fn with_id(&self, id: ID) -> &Self {
        self.set("session_id", id);
        self
    }

    pub fn id(&self) -> ID {
        self.get::<ID>("session_id").unwrap_or(0)
    }
}

/// ContextWithID returns a new context with the given ID.
pub fn context_with_id(ctx: &Context, id: ID) -> Context {
    let new_ctx = Context::new();
    // Copy existing keys if needed
    if let Ok(guard) = ctx.values.read()
        && let Ok(mut new_guard) = new_ctx.values.write()
    {
        for k in guard.keys() {
            if let Some(val) = ctx.get::<ID>(k) {
                new_guard.insert(k.clone(), Box::new(val));
            }
        }
    }
    new_ctx.set("session_id", id);
    new_ctx
}

/// IDFromContext returns ID in this context, or 0 if not contained.
pub fn id_from_context(ctx: &Context) -> ID {
    ctx.id()
}
