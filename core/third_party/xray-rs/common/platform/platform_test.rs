// Module: common\platform\platform_test.rs
// 1:1 Rust unit test suite corresponding to Go common\platform\platform_test.go

#[cfg(test)]
mod tests {
    use super::super::platform::*;
    use super::super::windows::is_windows;

    #[test]
    fn test_normalize_env_name() {
        let cases = vec![
            ("a", "A"),
            ("a.a", "A_A"),
            ("A.A.B", "A_A_B"),
            ("  xray.location.asset  ", "XRAY_LOCATION_ASSET"),
        ];

        for (input, expected) in cases {
            assert_eq!(normalize_env_name(input), expected);
        }
    }

    #[test]
    fn test_env_flag() {
        let flag = EnvFlag::new("xxxxx.y.nonexistent");
        assert_eq!(flag.get_value_as_int(10), 10);
        assert_eq!(flag.get_value_str("fallback"), "fallback");
        assert_eq!(flag.get_value_as_bool(true), true);
    }

    #[test]
    fn test_platform_detection() {
        let os = get_os_name();
        assert!(!os.is_empty());
        let asset = get_asset_location("geoip.dat");
        assert!(asset.to_string_lossy().contains("geoip.dat"));
        assert_eq!(is_windows(), cfg!(target_os = "windows"));
    }

    #[test]
    fn test_filesystem_operations() {
        use super::super::filesystem::{copy_file, file_exists, read_file, write_file};
        let temp_dir = std::env::temp_dir();
        let file_src = temp_dir.join("xray_test_fs_src.txt");
        let file_dst = temp_dir.join("xray_test_fs_dst.txt");

        let content = b"hello-xray-fs";
        write_file(&file_src, content).expect("write src");
        assert!(file_exists(&file_src));

        copy_file(&file_dst, &file_src).expect("copy file");
        assert!(file_exists(&file_dst));

        let read_back = read_file(&file_dst).expect("read dst");
        assert_eq!(read_back, content);

        let _ = std::fs::remove_file(file_src);
        let _ = std::fs::remove_file(file_dst);
    }
}
