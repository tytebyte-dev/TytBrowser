// Module: features\extension\observatory.rs
// 1:1 Rust implementation corresponding to Go features\extension\observatory.go

use crate::common::errors::Result;
use crate::features::feature::{Feature, TYPE_OBSERVATORY};
use async_trait::async_trait;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Observation {
    pub tag: String,
    pub latency: Option<Duration>,
    pub last_seen: u64,
}

#[async_trait]
pub trait ObservatoryFeature: Feature {
    async fn get_observation(&self, tag: &str) -> Result<Option<Observation>>;
    async fn list_observations(&self) -> Vec<Observation>;

    fn select_outbound(&self, tags: &[String]) -> Option<String> {
        tags.first().cloned()
    }
}

pub trait BurstObservatory: ObservatoryFeature {
    fn check(&self, tags: &[String]);
}

pub fn observatory_type() -> &'static str {
    TYPE_OBSERVATORY
}
