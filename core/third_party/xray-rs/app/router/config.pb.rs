// Module: app\router\config.pb.rs
// 1:1 Rust protobuf message definitions corresponding to Go app\router\config.pb.go

use super::webhook::WebhookConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DomainType {
    #[default]
    Plain = 0,
    Regex = 1,
    Domain = 2,
    Full = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DomainStrategy {
    #[default]
    AsIs = 0,
    IpIfNonMatch = 2,
    IpOnDemand = 3,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Domain {
    #[serde(rename = "type", default)]
    pub r#type: DomainType,
    #[serde(default)]
    pub value: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cidr {
    #[serde(default)]
    pub ip: Vec<u8>,
    #[serde(default)]
    pub prefix: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeoIP {
    #[serde(rename = "country_code", alias = "countryCode", default)]
    pub country_code: String,
    #[serde(default)]
    pub cidr: Vec<Cidr>,
    #[serde(rename = "reverse_match", alias = "reverseMatch", default)]
    pub reverse_match: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeoSite {
    #[serde(rename = "country_code", alias = "countryCode", default)]
    pub country_code: String,
    #[serde(default)]
    pub domain: Vec<Domain>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BalancerConfig {
    #[serde(default)]
    pub tag: String,
    #[serde(rename = "client_tags", alias = "clientTags", default)]
    pub client_tags: Vec<String>,
    #[serde(rename = "fallback_tag", alias = "fallbackTag", default)]
    pub fallback_tag: Option<String>,
    #[serde(default)]
    pub strategy: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingRule {
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(rename = "balancing_tag", alias = "balancingTag", default)]
    pub balancing_tag: Option<String>,
    #[serde(rename = "rule_tag", alias = "ruleTag", default)]
    pub rule_tag: Option<String>,
    #[serde(default)]
    pub domain: Vec<Domain>,
    #[serde(default)]
    pub geoip: Vec<GeoIP>,
    #[serde(default)]
    pub networks: Vec<String>,
    #[serde(rename = "source_geoip", alias = "sourceGeoip", default)]
    pub source_geoip: Vec<GeoIP>,
    #[serde(rename = "user_email", alias = "userEmail", default)]
    pub user_email: Vec<String>,
    #[serde(rename = "inbound_tag", alias = "inboundTag", default)]
    pub inbound_tag: Vec<String>,
    #[serde(default)]
    pub protocol: Vec<String>,
    #[serde(default)]
    pub attributes: HashMap<String, String>,
    #[serde(default)]
    pub process: Vec<String>,
    #[serde(default)]
    pub webhook: Option<WebhookConfig>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "domain_strategy", alias = "domainStrategy", default)]
    pub domain_strategy: DomainStrategy,
    #[serde(rename = "routing_rule", alias = "routingRule", default)]
    pub routing_rule: Vec<RoutingRule>,
    #[serde(default)]
    pub balancers: Vec<BalancerConfig>,
}
