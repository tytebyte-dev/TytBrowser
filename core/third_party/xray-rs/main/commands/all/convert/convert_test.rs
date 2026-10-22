// Module: main\commands\all\convert\convert_test.rs

#[cfg(test)]
mod tests {
    use super::super::convert::cmd_convert;
    use super::super::json::format_json;
    use super::super::protobuf::convert_configs_to_pb_json;

    #[test]
    fn test_format_json() {
        let raw = r#"{"a":1}"#;
        let formatted = format_json(raw).unwrap();
        assert!(formatted.contains('\n'));
    }

    #[test]
    fn test_convert_pb_merge() {
        let raw1 = r#"{"inbounds": [{"protocol": "socks"}]}"#;
        let raw2 = r#"{"outbounds": [{"protocol": "freedom"}]}"#;
        let merged = convert_configs_to_pb_json(&[raw1, raw2]).unwrap();
        assert!(merged.contains("socks"));
        assert!(merged.contains("freedom"));
    }

    #[test]
    fn test_convert_subcommands() {
        let cmd = cmd_convert();
        assert_eq!(cmd.name, "convert");
        assert_eq!(cmd.subcommands.len(), 2);

        let res = cmd.execute(&["json", r#"{"x": 10}"#]).unwrap();
        assert!(res.contains("\"x\": 10"));
    }
}
