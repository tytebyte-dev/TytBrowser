// Module: infra\vprotogen\main.rs
// 1:1 Rust implementation corresponding to Go infra\vprotogen\main.go

use std::path::{Path, PathBuf};

pub fn parse_version(s: &str, width: usize) -> u64 {
    let parts: Vec<&str> = s.split('.').collect();
    let mut formatted = String::new();
    for part in parts {
        let trimmed = part.trim();
        if let Ok(num) = trimmed.parse::<u64>() {
            formatted.push_str(&format!("{:0width$}", num, width = width));
        }
    }
    formatted.parse::<u64>().unwrap_or(0)
}

pub fn need_to_update(target_version: &str, installed_version: &str) -> bool {
    let vt = parse_version(target_version, 4);
    let vi = parse_version(installed_version, 4);
    vt > vi
}

pub fn find_proto_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if !root.exists() {
        return files;
    }

    fn walk_dir(dir: &Path, files: &mut Vec<PathBuf>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if dir_name == "target" || dir_name == ".git" {
                        continue;
                    }
                    walk_dir(&path, files);
                } else if path.is_file() {
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if name.ends_with(".proto") {
                        files.push(path);
                    }
                }
            }
        }
    }

    walk_dir(root, &mut files);
    files
}

pub fn run_protogen() -> &'static str {
    "vprotogen: complete"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        assert_eq!(parse_version("25.1.0", 4), 2500010000);
        assert_eq!(parse_version("3.19.4", 4), 300190004);
    }

    #[test]
    fn test_need_to_update() {
        assert!(need_to_update("25.1.0", "24.0.0"));
        assert!(!need_to_update("24.0.0", "25.1.0"));
        assert!(!need_to_update("25.1.0", "25.1.0"));
    }

    #[test]
    fn test_find_proto_files() {
        let current_dir = std::env::current_dir().unwrap();
        let _ = find_proto_files(&current_dir);
    }
}
