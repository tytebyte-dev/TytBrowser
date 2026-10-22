// Module: common\singbridge\error.rs
// 1:1 Rust implementation corresponding to Go common\singbridge\error.go

use crate::common::errors::Error;
use std::io;

/// IsClosedOrCanceled returns true if the error indicates a closed connection or canceled operation.
/// 1:1 corresponding to exceptions.IsClosedOrCanceled() in sing.
pub fn is_closed_or_canceled(err: &Error) -> bool {
    match err {
        Error::Io(e) => {
            matches!(
                e.kind(),
                io::ErrorKind::BrokenPipe
                    | io::ErrorKind::ConnectionReset
                    | io::ErrorKind::ConnectionAborted
                    | io::ErrorKind::NotConnected
                    | io::ErrorKind::UnexpectedEof
                    | io::ErrorKind::Interrupted
            )
        }
        Error::Other(msg) => {
            let m = msg.to_lowercase();
            m.contains("closed")
                || m.contains("canceled")
                || m.contains("cancelled")
                || m.contains("broken pipe")
                || m.contains("eof")
                || m.contains("connection reset")
        }
        _ => false,
    }
}

/// ReturnError returns None if err indicates a closed or canceled connection, or the error itself otherwise.
/// 1:1 corresponding to ReturnError() in error.go.
pub fn return_error(err: Option<Error>) -> Option<Error> {
    match err {
        Some(e) if is_closed_or_canceled(&e) => None,
        other => other,
    }
}

/// Helper to wrap Result, converting closed/canceled errors into Ok(None).
pub fn return_result<T>(res: Result<T, Error>) -> Result<Option<T>, Error> {
    match res {
        Ok(v) => Ok(Some(v)),
        Err(e) if is_closed_or_canceled(&e) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn wrap_sing_error(err_str: impl Into<String>) -> Error {
    Error::Protocol(format!("sing-box bridge error: {}", err_str.into()))
}
