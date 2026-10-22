#[cfg(test)]
mod tests {
    use crate::infra::conf::xray::ConfigLoader;

    #[test]
    fn test_config_loader_full_json() {
        let json_data = r#"{
            "log": {
                "loglevel": "warning"
            },
            "inbounds": [
                {
                    "tag": "socks-in",
                    "port": 10808,
                    "listen": "127.0.0.1",
                    "protocol": "socks",
                    "settings": {
                        "auth": "noauth",
                        "udp": true
                    }
                }
            ],
            "outbounds": [
                {
                    "tag": "direct",
                    "protocol": "freedom",
                    "settings": {}
                },
                {
                    "tag": "blocked",
                    "protocol": "blackhole",
                    "settings": {}
                }
            ],
            "routing": {
                "domainStrategy": "AsIs",
                "rules": [
                    {
                        "type": "field",
                        "outboundTag": "blocked",
                        "domain": ["geosite:category-ads-all"]
                    },
                    {
                        "type": "field",
                        "outboundTag": "direct",
                        "network": "tcp,udp"
                    }
                ]
            }
        }"#;

        let config = ConfigLoader::load_from_json_str(json_data).unwrap();
        assert_eq!(config.log.unwrap().loglevel, Some("warning".to_string()));
        assert_eq!(config.inbounds.len(), 1);
        assert_eq!(config.inbounds[0].tag, Some("socks-in".to_string()));
        assert_eq!(config.inbounds[0].port, Some(10808));
        assert_eq!(config.outbounds.len(), 2);
        assert_eq!(config.outbounds[0].tag, Some("direct".to_string()));
        assert_eq!(config.outbounds[1].tag, Some("blocked".to_string()));

        let routing = config.routing.unwrap();
        let rules = routing.rules.unwrap();
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].outbound_tag, "blocked");
    }
}
