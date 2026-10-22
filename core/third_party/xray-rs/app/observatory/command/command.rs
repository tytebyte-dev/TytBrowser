// Module: app\observatory\command\command.rs
// 1:1 Rust implementation corresponding to Go app\observatory\command\command.go

use super::command_grpc_pb::ObservatoryService as IObservatoryService;
use super::command_pb::{GetOutboundStatusRequest, GetOutboundStatusResponse};
use crate::app::observatory::Observatory;
use crate::app::observatory::config_pb::{
    ObservationResult, OutboundStatus as ProtoOutboundStatus,
};
use crate::common::errors::Result;
use async_trait::async_trait;
use std::sync::Arc;

pub struct ObservatoryCommandServer {
    observatory: Arc<Observatory>,
}

impl ObservatoryCommandServer {
    pub fn new(observatory: Arc<Observatory>) -> Self {
        Self { observatory }
    }

    pub fn get_outbound_status(&self, tag: &str) -> Option<bool> {
        self.observatory.get_status(tag).map(|s| s.alive)
    }
}

#[async_trait]
impl IObservatoryService for ObservatoryCommandServer {
    async fn get_outbound_status(
        &self,
        _req: GetOutboundStatusRequest,
    ) -> Result<GetOutboundStatusResponse> {
        let statuses = self.observatory.all_statuses();
        let proto_statuses = statuses
            .into_iter()
            .map(|s| ProtoOutboundStatus {
                alive: s.alive,
                delay: s.delay_ms as i64,
                last_error_reason: s.last_error_reason.unwrap_or_default(),
                outbound_tag: s.outbound_tag,
                last_seen_time: s.last_seen_time as i64,
                last_try_time: s.last_try_time as i64,
                health_ping: None,
            })
            .collect();

        Ok(GetOutboundStatusResponse {
            status: Some(ObservationResult {
                status: proto_statuses,
            }),
        })
    }
}
