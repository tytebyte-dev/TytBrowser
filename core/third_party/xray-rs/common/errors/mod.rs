pub mod errors;
pub mod feature_errors;
pub mod multi_error;

#[cfg(test)]
pub mod errors_test;

use thiserror::Error;

pub use errors::{
    ErrorDetail, cause, get_severity, log_debug, log_debug_inner, log_error, log_error_inner,
    log_info, log_info_inner, log_warning, log_warning_inner, new as new_error_detail, new_error,
};
pub use feature_errors::{
    missing_feature, print_deprecated_feature_warning,
    print_non_removal_deprecated_feature_warning, print_removed_feature_error,
};
pub use multi_error::{MultiError, all_equal, combine};

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Address parse error: {0}")]
    AddressParse(String),

    #[error("Invalid protocol: {0}")]
    Protocol(String),

    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Unsupported feature: {0}")]
    Unsupported(String),

    #[error("Handler not found: {0}")]
    NotFound(String),

    #[error("Connection closed")]
    Closed,

    #[error("End of file")]
    Eof,

    #[error("Timeout")]
    Timeout,

    #[error("Buffer overflow")]
    BufferOverflow,

    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
