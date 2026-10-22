// Module: features\routing\router.rs
// 1:1 Rust implementation corresponding to Go features\routing\router.go

use super::context::RoutingContext;
use crate::common::errors::{Error, Result};
use crate::common::protocol::SessionContext;
use crate::features::feature::{Feature, TYPE_ROUTER};
use async_trait::async_trait;

pub trait Route: Send + Sync {
    fn outbound_tag(&self) -> &str;
    fn rule_tag(&self) -> Option<&str> {
        None
    }
    fn outbound_group_tags(&self) -> &[String] {
        &[]
    }
}

#[derive(Debug, Clone)]
pub struct DefaultRoute {
    pub tag: String,
    pub rule: Option<String>,
    pub group_tags: Vec<String>,
}

impl Route for DefaultRoute {
    fn outbound_tag(&self) -> &str {
        &self.tag
    }
    fn rule_tag(&self) -> Option<&str> {
        self.rule.as_deref()
    }
    fn outbound_group_tags(&self) -> &[String] {
        &self.group_tags
    }
}

#[async_trait]
pub trait Router: Feature {
    async fn pick_route(&self, ctx: &dyn RoutingContext) -> Result<String>;
    async fn list_rules(&self) -> Vec<String> {
        Vec::new()
    }
}

pub trait RouterFeature: Send + Sync {
    fn pick_outbound(&self, session: &SessionContext) -> Option<&str>;
}

#[derive(Default, Clone, Debug)]
pub struct DefaultRouter;

impl Feature for DefaultRouter {
    fn feature_type(&self) -> &'static str {
        TYPE_ROUTER
    }
}

#[async_trait]
impl Router for DefaultRouter {
    async fn pick_route(&self, _ctx: &dyn RoutingContext) -> Result<String> {
        Err(Error::NotFound("No matching route".into()))
    }
}

impl RouterFeature for DefaultRouter {
    fn pick_outbound(&self, _session: &SessionContext) -> Option<&str> {
        None
    }
}

pub fn router_type() -> &'static str {
    TYPE_ROUTER
}
