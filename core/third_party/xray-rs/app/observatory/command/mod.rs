// Module: app\observatory\command\mod.rs

pub mod command;
#[path = "command_grpc.pb.rs"]
pub mod command_grpc_pb;
#[path = "command.pb.rs"]
pub mod command_pb;

#[cfg(test)]
pub mod command_test;

use crate::app::commander::Service;
use crate::app::observatory::{Observatory, OutboundStatus};
use std::sync::Arc;

pub use command::ObservatoryCommandServer;
pub use command_grpc_pb::ObservatoryService as IObservatoryService;
pub use command_pb::*;

pub struct ObservatoryService {
    observatory: Arc<Observatory>,
}

impl ObservatoryService {
    pub fn new(observatory: Arc<Observatory>) -> Self {
        Self { observatory }
    }

    pub fn get_outbound_status(&self, tag: &str) -> Option<OutboundStatus> {
        self.observatory.get_status(tag)
    }

    pub fn get_all_statuses(&self) -> Vec<OutboundStatus> {
        self.observatory.all_statuses()
    }
}

impl Service for ObservatoryService {
    fn service_name(&self) -> &str {
        "xray.core.app.observatory.command.ObservatoryService"
    }
}
