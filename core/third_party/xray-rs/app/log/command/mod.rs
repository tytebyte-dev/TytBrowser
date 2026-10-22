pub mod command;

#[cfg(test)]
pub mod command_test;

use crate::app::commander::Service;
use crate::app::log::LogManager;
use crate::common::errors::Result;
use std::sync::Arc;

pub use command::LoggerServer;

pub struct LoggerService {
    log_manager: Arc<LogManager>,
}

impl LoggerService {
    pub fn new(log_manager: Arc<LogManager>) -> Self {
        Self { log_manager }
    }

    pub fn restart_logger(&self) -> Result<()> {
        self.log_manager.start();
        Ok(())
    }
}

impl Service for LoggerService {
    fn service_name(&self) -> &str {
        "xray.core.app.log.command.LoggerService"
    }
}
