// Module: common\platform\others.rs
// 1:1 Rust implementation corresponding to Go common\platform\others.go

use crate::common::platform::platform::{
    ASSET_LOCATION, CERT_LOCATION, EnvFlag, get_executable_dir,
};
use std::path::PathBuf;

pub fn line_separator() -> &'static str {
    "\n"
}

pub fn get_asset_location(file: &str) -> PathBuf {
    let flag = EnvFlag::new(ASSET_LOCATION);
    let asset_path = flag.get_value(|| get_executable_dir().to_string_lossy().into_owned());
    let def_path = PathBuf::from(asset_path).join(file);

    let candidates = [
        def_path.clone(),
        PathBuf::from("/usr/local/share/xray/").join(file),
        PathBuf::from("/usr/share/xray/").join(file),
        PathBuf::from("/opt/share/xray/").join(file),
    ];

    for p in &candidates {
        if p.exists() {
            return p.clone();
        }
    }

    def_path
}

pub fn get_cert_location(file: &str) -> PathBuf {
    let flag = EnvFlag::new(CERT_LOCATION);
    let cert_path = flag.get_value(|| get_executable_dir().to_string_lossy().into_owned());
    PathBuf::from(cert_path).join(file)
}
