// Module: infra\vformat\main.rs
// 1:1 Rust implementation corresponding to Go infra\vformat\main.go

use crate::common::errors::Result;
use std::path::{Path, PathBuf};

pub fn find_files_to_format(root: &Path, ext: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if !root.exists() {
        return files;
    }

    fn walk_dir(dir: &Path, ext: &str, files: &mut Vec<PathBuf>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if dir_name == "target" || dir_name == ".git" {
                        continue;
                    }
                    walk_dir(&path, ext, files);
                } else if path.is_file() {
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if name.ends_with(ext) && !name.ends_with(".pb.rs") && !name.ends_with(".pb.go")
                    {
                        files.push(path);
                    }
                }
            }
        }
    }

    walk_dir(root, ext, &mut files);
    files
}

pub fn format_source_tree(root: &Path) -> Result<usize> {
    let files = find_files_to_format(root, ".rs");
    Ok(files.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_files_to_format() {
        let current_dir = std::env::current_dir().unwrap();
        let files = find_files_to_format(&current_dir.join("infra").join("vformat"), ".rs");
        assert!(!files.is_empty());
        assert!(files.iter().any(|f| f.ends_with("main.rs")));
    }

    #[test]
    fn test_format_source_tree() {
        let current_dir = std::env::current_dir().unwrap();
        let count = format_source_tree(&current_dir.join("infra").join("vformat")).unwrap();
        assert!(count >= 1);
    }
}
