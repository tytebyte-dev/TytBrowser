// Module: transport\internet\filelocker.rs
// 1:1 Rust implementation corresponding to Go transport\internet\filelocker.go

use crate::common::errors::Result;
use std::fs::File;
use std::path::{Path, PathBuf};

pub struct FileLocker {
    pub path: PathBuf,
    file: Option<File>,
}

impl FileLocker {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            file: None,
        }
    }

    #[cfg(windows)]
    pub fn acquire(&mut self) -> Result<()> {
        let f = File::create(&self.path)?;
        self.file = Some(f);
        Ok(())
    }

    #[cfg(not(windows))]
    pub fn acquire(&mut self) -> Result<()> {
        let f = File::create(&self.path)?;
        self.file = Some(f);
        Ok(())
    }

    pub fn release(&mut self) {
        if self.file.take().is_some() {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

impl Drop for FileLocker {
    fn drop(&mut self) {
        self.release();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_locker_lifecycle() {
        let temp_dir = std::env::temp_dir();
        let lock_path = temp_dir.join("xray_test_lock.lock");
        let mut locker = FileLocker::new(&lock_path);
        assert!(locker.acquire().is_ok());
        assert!(lock_path.exists());
        locker.release();
        assert!(!lock_path.exists());
    }
}
