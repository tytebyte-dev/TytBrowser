pub mod bytesize;

#[cfg(test)]
pub mod bytesize_test;

pub use crate::common::errors::{Error, Result};
pub use bytesize::{ByteSize, EB, GB, KB, MB, PB, TB, format_bytes};

pub fn parse_bytesize(s: &str) -> Result<u64> {
    let mut bs = ByteSize::default();
    bs.parse(s)?;
    Ok(bs.0)
}

pub fn format_bytesize(bytes: u64) -> String {
    ByteSize(bytes).to_string_formatted()
}
