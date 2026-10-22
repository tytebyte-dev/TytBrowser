// Module: core\context.rs
// 1:1 Rust implementation corresponding to Go core\context.go

use super::xray::Instance;
use crate::common::errors::{Error, Result};
use std::sync::Arc;

pub const XRAY_KEY: &str = "xray_instance";

#[derive(Clone)]
pub struct CoreContext {
    instance: Option<Arc<Instance>>,
}

impl CoreContext {
    pub fn new() -> Self {
        Self { instance: None }
    }

    pub fn with_instance(instance: Arc<Instance>) -> Self {
        Self {
            instance: Some(instance),
        }
    }

    pub fn get_instance(&self) -> Option<Arc<Instance>> {
        self.instance.clone()
    }

    pub fn must_get_instance(&self) -> Result<Arc<Instance>> {
        self.instance
            .clone()
            .ok_or_else(|| Error::Other("Instance is not in context".into()))
    }

    pub fn to_background_detached_context(&self) -> Result<Self> {
        let inst = self.must_get_instance()?;
        Ok(Self::with_instance(inst))
    }
}

impl Default for CoreContext {
    fn default() -> Self {
        Self::new()
    }
}

pub fn from_context(ctx: &CoreContext) -> Option<Arc<Instance>> {
    ctx.get_instance()
}

pub fn must_from_context(ctx: &CoreContext) -> Result<Arc<Instance>> {
    ctx.must_get_instance()
}

pub fn to_context(ctx: &mut CoreContext, instance: Arc<Instance>) {
    *ctx = CoreContext::with_instance(instance);
}

pub fn to_background_detached_context(ctx: &CoreContext) -> Result<CoreContext> {
    ctx.to_background_detached_context()
}
