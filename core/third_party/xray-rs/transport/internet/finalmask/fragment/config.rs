// Module: transport\internet\finalmask\fragment\config.rs
// 1:1 Rust implementation corresponding to Go transport\internet\finalmask\fragment\config.go

pub use super::config_pb::Config;
use super::conn::FragmentConn;
use crate::common::errors::Result;
use crate::common::net::BoxStream;

impl Config {
    pub fn tcp(&self) {}

    pub fn wrap_conn_client(&self, raw: BoxStream) -> Result<FragmentConn> {
        Ok(FragmentConn::new_client(self.clone(), raw))
    }

    pub fn wrap_conn_server(&self, raw: BoxStream) -> Result<FragmentConn> {
        Ok(FragmentConn::new_server(self.clone(), raw))
    }
}
