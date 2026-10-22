pub mod balancing;
pub mod balancing_override;
pub mod command;
pub mod condition;
pub mod condition_geoip;
pub mod config;
#[path = "config.pb.rs"]
pub mod config_pb;
pub mod geo;
pub mod geosite_compact;
pub mod router;
pub mod strategy_leastload;
pub mod strategy_leastping;
pub mod strategy_random;
pub mod webhook;
pub mod weight;

#[cfg(test)]
pub mod condition_geoip_test;
#[cfg(test)]
pub mod condition_serialize_test;
#[cfg(test)]
pub mod condition_test;
#[cfg(test)]
pub mod router_test;
#[cfg(test)]
pub mod strategy_leastload_test;
#[cfg(test)]
pub mod weight_test;

pub use balancing::{Balancer, BalancingStrategy, RoundRobinStrategy};
pub use balancing_override::BalancingOverride;
pub use condition::{DomainMatcher, IpMatcher, Rule};
pub use condition_geoip::{Cidr, GeoIpMatcher, HeuristicGeoIpMatcher};
pub use config::RouterConfig;
pub use geo::{GeoDatabase, GeoIpGroup, GeoSiteGroup};
pub use geosite_compact::GeoSiteCompactList;
pub use router::Router;
pub use strategy_leastload::LeastLoadStrategy;
pub use strategy_leastping::LeastPingStrategy;
pub use strategy_random::RandomStrategy;
pub use webhook::{WebhookConfig, WebhookEvent, WebhookNotifier, parse_url, resolve_socket_path};
pub use weight::{StrategyWeight, WeightManager};
