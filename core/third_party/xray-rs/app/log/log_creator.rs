use crate::app::log::LogLevel;
use crate::common::errors::{Error, Result};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogType {
    Console,
    File,
    None,
}

pub struct LogCreatorOptions {
    pub log_type: LogType,
    pub path: Option<String>,
    pub level: LogLevel,
}

impl LogCreatorOptions {
    pub fn console(level: LogLevel) -> Self {
        Self {
            log_type: LogType::Console,
            path: None,
            level,
        }
    }

    pub fn file(path: impl Into<String>, level: LogLevel) -> Self {
        Self {
            log_type: LogType::File,
            path: Some(path.into()),
            level,
        }
    }

    pub fn none() -> Self {
        Self {
            log_type: LogType::None,
            path: None,
            level: LogLevel::None,
        }
    }
}

pub fn create_logger(options: &LogCreatorOptions) -> Result<()> {
    match options.log_type {
        LogType::Console => Ok(()),
        LogType::File => {
            if let Some(path) = &options.path {
                if let Some(parent) = Path::new(path).parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                Ok(())
            } else {
                Err(Error::Config("File logger requires path".into()))
            }
        }
        LogType::None => Ok(()),
    }
}
