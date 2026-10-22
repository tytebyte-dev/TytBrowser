// Module: common\errors\errors.rs
// 1:1 Rust implementation corresponding to Go common\errors\errors.go

pub use super::{Error, Result};
use crate::common::log::Severity;
use std::fmt;

/// ErrorDetail represents an error with contextual prefixes, caller information,
/// severity levels, and chained inner causes, corresponding to Go `errors.Error`.
#[derive(Debug, Clone)]
pub struct ErrorDetail {
    pub prefix: Vec<String>,
    pub message: String,
    pub caller: String,
    pub inner: Option<Box<ErrorDetail>>,
    pub severity: Severity,
}

impl ErrorDetail {
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            prefix: Vec::new(),
            message: msg.into(),
            caller: String::new(),
            inner: None,
            severity: Severity::Info,
        }
    }

    pub fn with_caller(msg: impl Into<String>, caller: impl Into<String>) -> Self {
        Self {
            prefix: Vec::new(),
            message: msg.into(),
            caller: caller.into(),
            inner: None,
            severity: Severity::Info,
        }
    }

    pub fn base(mut self, inner: ErrorDetail) -> Self {
        self.inner = Some(Box::new(inner));
        self
    }

    pub fn at_severity(mut self, s: Severity) -> Self {
        self.severity = s;
        self
    }

    pub fn at_debug(self) -> Self {
        self.at_severity(Severity::Debug)
    }

    pub fn at_info(self) -> Self {
        self.at_severity(Severity::Info)
    }

    pub fn at_warning(self) -> Self {
        self.at_severity(Severity::Warning)
    }

    pub fn at_error(self) -> Self {
        self.at_severity(Severity::Error)
    }

    pub fn severity(&self) -> Severity {
        if let Some(ref inner) = self.inner {
            let inner_sev = inner.severity();
            // Lower severity numeric value in log.pb represents higher priority (Error=1, Warning=2, Info=3)
            let cur = self.severity as u8;
            let inn = inner_sev as u8;
            if inn > 0 && (cur == 0 || inn < cur) {
                return inner_sev;
            }
        }
        self.severity
    }

    pub fn unwrap(&self) -> Option<&ErrorDetail> {
        self.inner.as_deref()
    }
}

impl fmt::Display for ErrorDetail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for p in &self.prefix {
            write!(f, "[{}] ", p)?;
        }
        if !self.caller.is_empty() {
            write!(f, "{}: ", self.caller)?;
        }
        write!(f, "{}", self.message)?;
        if let Some(ref inner) = self.inner {
            write!(f, " > {}", inner)?;
        }
        Ok(())
    }
}

impl std::error::Error for ErrorDetail {}

impl From<ErrorDetail> for Error {
    fn from(detail: ErrorDetail) -> Self {
        Error::Other(detail.to_string())
    }
}

pub fn new_error(msg: impl Into<String>) -> Error {
    Error::Other(msg.into())
}

pub fn new(msg: impl Into<String>) -> ErrorDetail {
    ErrorDetail::new(msg)
}

/// Cause returns the root cause of this error by traversing inner errors.
pub fn cause(mut err: &ErrorDetail) -> &ErrorDetail {
    while let Some(ref inner) = err.inner {
        err = inner;
    }
    err
}

/// GetSeverity returns the actual severity of the error, including inner errors.
pub fn get_severity(err: &ErrorDetail) -> Severity {
    err.severity()
}

pub fn log_debug(msg: impl fmt::Display) {
    tracing::debug!("{}", msg);
}

pub fn log_debug_inner(inner: impl fmt::Display, msg: impl fmt::Display) {
    tracing::debug!("{} > {}", msg, inner);
}

pub fn log_info(msg: impl fmt::Display) {
    tracing::info!("{}", msg);
}

pub fn log_info_inner(inner: impl fmt::Display, msg: impl fmt::Display) {
    tracing::info!("{} > {}", msg, inner);
}

pub fn log_warning(msg: impl fmt::Display) {
    tracing::warn!("{}", msg);
}

pub fn log_warning_inner(inner: impl fmt::Display, msg: impl fmt::Display) {
    tracing::warn!("{} > {}", msg, inner);
}

pub fn log_error(msg: impl fmt::Display) {
    tracing::error!("{}", msg);
}

pub fn log_error_inner(inner: impl fmt::Display, msg: impl fmt::Display) {
    tracing::error!("{} > {}", msg, inner);
}
