// Module: app\observatory\command\command_grpc.pb.rs
use super::command_pb::{GetOutboundStatusRequest, GetOutboundStatusResponse};
use crate::common::errors::Result;
use async_trait::async_trait;

#[async_trait]
pub trait ObservatoryService: Send + Sync {
    async fn get_outbound_status(
        &self,
        req: GetOutboundStatusRequest,
    ) -> Result<GetOutboundStatusResponse>;
}
