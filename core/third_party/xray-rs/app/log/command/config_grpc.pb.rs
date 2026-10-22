// Module: app\log\command\config_grpc.pb.rs
use async_trait::async_trait;
use crate::common::errors::Result;
use super::config_pb::{RestartLoggerRequest, RestartLoggerResponse};

#[async_trait]
pub trait LoggerService: Send + Sync {
    async fn restart_logger(&self, req: RestartLoggerRequest) -> Result<RestartLoggerResponse>;
}
