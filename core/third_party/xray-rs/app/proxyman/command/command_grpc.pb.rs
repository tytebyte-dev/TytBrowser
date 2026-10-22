// Module: app\proxyman\command\command_grpc.pb.rs
use async_trait::async_trait;
use crate::common::errors::Result;
use super::command_pb::*;

#[async_trait]
pub trait HandlerService: Send + Sync {
    async fn add_inbound(&self, req: AddInboundRequest) -> Result<AddInboundResponse>;
    async fn remove_inbound(&self, req: RemoveInboundRequest) -> Result<RemoveInboundResponse>;
    async fn alter_inbound(&self, req: AlterInboundRequest) -> Result<AlterInboundResponse>;
    async fn add_outbound(&self, req: AddOutboundRequest) -> Result<AddOutboundResponse>;
    async fn remove_outbound(&self, req: RemoveOutboundRequest) -> Result<RemoveOutboundResponse>;
    async fn alter_outbound(&self, req: AlterOutboundRequest) -> Result<AlterOutboundResponse>;
}
