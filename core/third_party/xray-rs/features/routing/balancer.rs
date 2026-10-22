// Module: features\routing\balancer.rs
// 1:1 Rust implementation corresponding to Go features\routing\balancer.go

use crate::common::errors::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Balancer: Send + Sync {
    fn tag(&self) -> &str;
    fn pick_outbound(&self) -> Option<String>;
}

pub trait BalancerOverrider: Send + Sync {
    fn set_override_target(&self, tag: &str, target: &str) -> Result<()>;
    fn get_override_target(&self, tag: &str) -> Result<String>;
}

pub trait BalancerPrincipleTarget: Send + Sync {
    fn get_principle_target(&self, tag: &str) -> Result<Vec<String>>;
}

pub trait BalancerFeature: Send + Sync {
    fn pick_outbound(&self, candidates: &[String]) -> Option<String>;
}
