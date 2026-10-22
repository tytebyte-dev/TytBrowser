// Module: app\stats\command\command_grpc.pb.rs
use async_trait::async_trait;
use crate::common::errors::Result;
use super::command_pb::*;

#[async_trait]
pub trait StatsService: Send + Sync {
    async fn get_stats(&self, req: GetStatsRequest) -> Result<GetStatsResponse>;
    async fn query_stats(&self, req: QueryStatsRequest) -> Result<QueryStatsResponse>;
    async fn get_sys_stats(&self, req: GetSysStatsRequest) -> Result<SysStatsResponse>;
}
