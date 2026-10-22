// Module: common\errors\multi_error.rs
// 1:1 Rust implementation corresponding to Go common\errors\multi_error.go

use crate::common::errors::Error;
use std::fmt;

#[derive(Debug, Default)]
pub struct MultiError {
    errors: Vec<Error>,
}

impl MultiError {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn add(&mut self, err: Error) {
        self.errors.push(err);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn len(&self) -> usize {
        self.errors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn errors(&self) -> &[Error] {
        &self.errors
    }

    pub fn into_result(self) -> Result<(), Error> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(Error::Other(self.to_string()))
        }
    }
}

impl fmt::Display for MultiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "multierr: ")?;
        for err in &self.errors {
            write!(f, "{} | ", err)?;
        }
        Ok(())
    }
}

impl std::error::Error for MultiError {}

/// Combines multiple optional errors into a single MultiError or None.
pub fn combine(maybe_error: Vec<Option<Error>>) -> Option<MultiError> {
    let mut errs = MultiError::new();
    for e in maybe_error.into_iter().flatten() {
        errs.add(e);
    }
    if errs.is_empty() { None } else { Some(errs) }
}

/// Checks if all errors in MultiError match the expected error string.
pub fn all_equal(expected: &str, multi: &MultiError) -> bool {
    if multi.is_empty() {
        return false;
    }
    for err in multi.errors() {
        if !err.to_string().contains(expected) {
            return false;
        }
    }
    true
}
