// Module: common\platform\filesystem\file.rs
// 1:1 Rust implementation corresponding to Go common\platform\filesystem\file.go

use crate::common::errors::{Error, Result};
use crate::common::platform::platform::{get_asset_location, get_cert_location};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

pub fn file_exists(path: impl AsRef<Path>) -> bool {
    path.as_ref().exists()
}

pub fn read_file(path: impl AsRef<Path>) -> Result<Vec<u8>> {
    fs::read(path.as_ref()).map_err(Error::Io)
}

pub fn open_file(path: impl AsRef<Path>) -> Result<File> {
    File::open(path.as_ref()).map_err(Error::Io)
}

pub fn read_asset(file: &str) -> Result<Vec<u8>> {
    read_file(get_asset_location(file))
}

pub fn open_asset(file: &str) -> Result<File> {
    open_file(get_asset_location(file))
}

pub fn read_cert(file: &str) -> Result<Vec<u8>> {
    let path = PathBuf::from(file);
    if path.is_absolute() {
        return read_file(path);
    }
    read_file(get_cert_location(file))
}

pub fn copy_file(dst: impl AsRef<Path>, src: impl AsRef<Path>) -> Result<u64> {
    fs::copy(src.as_ref(), dst.as_ref()).map_err(Error::Io)
}

pub fn write_file(path: impl AsRef<Path>, data: &[u8]) -> Result<()> {
    if let Some(parent) = path.as_ref().parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent).map_err(Error::Io)?;
    }
    fs::write(path.as_ref(), data).map_err(Error::Io)
}

pub fn read_all_to_bytes<R: Read>(mut reader: R) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).map_err(Error::Io)?;
    Ok(buf)
}
