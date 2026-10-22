// Module: app\router\command\command_grpc.pb.rs
use async_trait::async_trait;
use crate::common::errors::Result;
use super::command_pb::*;

#[async_trait]
pub trait RoutingService: Send + Sync {
    async fn test_route(&self, ctx: RoutingContext) -> Result<RoutingContext>;
}
