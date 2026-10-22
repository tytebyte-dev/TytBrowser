// Module: core\xray_test.rs
// 1:1 Rust unit test suite corresponding to Go core\xray_test.go

#[cfg(test)]
mod tests {
    use crate::core::annotations::{Annotation, ApiStability};
    use crate::core::config::{ConfigSource, get_extension, get_format, get_format_by_extension};
    use crate::core::config_pb::{Config as PbConfig, InboundHandlerConfig, OutboundHandlerConfig};
    use crate::core::core::{VERSION_X, VERSION_Y, VERSION_Z, version, version_statement};
    use crate::core::xray::{Instance, server_type};
    use crate::infra::conf::Config;

    #[test]
    fn test_version_info() {
        assert_eq!(VERSION_X, 26);
        assert_eq!(VERSION_Y, 3);
        assert_eq!(VERSION_Z, 27);
        assert_eq!(version(), "26.3.27");
        let stmt = version_statement();
        assert_eq!(stmt.len(), 2);
        assert!(stmt[0].starts_with("Xray 26.3.27"));
        assert_eq!(stmt[1], "A unified platform for anti-censorship.");
    }

    #[test]
    fn test_xray_instance_lifecycle() {
        let config: Config = serde_json::from_str(
            r#"{
            "inbounds": [],
            "outbounds": [
                {
                    "protocol": "freedom"
                }
            ]
        }"#,
        )
        .unwrap();

        let instance = Instance::new(config).unwrap();
        assert_eq!(instance.type_name(), server_type());
        assert!(!instance.is_running());
        assert!(instance.close().is_ok());
        assert!(!instance.is_running());
    }

    #[test]
    fn test_config_source_and_format_resolution() {
        let src = ConfigSource::new("config.json", "json");
        assert_eq!(src.name, "config.json");
        assert_eq!(src.format, "json");

        assert_eq!(get_extension("foo/bar.json"), "json");
        assert_eq!(get_extension("baz.pb"), "pb");
        assert_eq!(get_extension("noext"), "");

        assert_eq!(get_format_by_extension("json"), "json");
        assert_eq!(get_format_by_extension("JSONC"), "json");
        assert_eq!(get_format_by_extension("yaml"), "yaml");
        assert_eq!(get_format_by_extension("yml"), "yaml");
        assert_eq!(get_format_by_extension("toml"), "toml");
        assert_eq!(get_format_by_extension("pb"), "protobuf");
        assert_eq!(get_format_by_extension("protobuf"), "protobuf");
        assert_eq!(get_format_by_extension("unknown"), "");

        assert_eq!(get_format("config.json"), "json");
        assert_eq!(get_format("config.yaml"), "yaml");
        assert_eq!(get_format("config.toml"), "toml");
        assert_eq!(get_format("config.pb"), "protobuf");
    }

    #[test]
    fn test_annotations_metadata() {
        let anno_stable = Annotation::from_stability(ApiStability::Stable);
        assert_eq!(anno_stable.api, "xray:api:stable");

        let anno_beta = Annotation::from_stability(ApiStability::Beta);
        assert_eq!(anno_beta.api, "xray:api:beta");

        let anno_custom = Annotation::new("xray:api:custom");
        assert_eq!(anno_custom.api, "xray:api:custom");
    }

    #[test]
    fn test_config_pb_structs() {
        let in_cfg = InboundHandlerConfig::new("inbound-1");
        assert_eq!(in_cfg.get_tag(), "inbound-1");
        assert!(in_cfg.get_receiver_settings().is_none());
        assert!(in_cfg.get_proxy_settings().is_none());

        let out_cfg = OutboundHandlerConfig::new("outbound-1");
        assert_eq!(out_cfg.get_tag(), "outbound-1");
        assert!(out_cfg.get_sender_settings().is_none());
        assert!(out_cfg.get_proxy_settings().is_none());

        let mut cfg = PbConfig::new();
        cfg.inbound.push(in_cfg);
        cfg.outbound.push(out_cfg);

        assert_eq!(cfg.get_inbound().len(), 1);
        assert_eq!(cfg.get_outbound().len(), 1);
        assert_eq!(cfg.get_app().len(), 0);
        assert_eq!(cfg.get_extension().len(), 0);

        let json = serde_json::to_string(&cfg).unwrap();
        let decoded: PbConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.inbound.len(), 1);
        assert_eq!(decoded.outbound.len(), 1);
    }
}
