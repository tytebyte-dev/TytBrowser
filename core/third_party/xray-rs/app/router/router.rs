use super::balancing::Balancer;
use super::condition::Rule;
use crate::common::errors::{Error, Result};
use crate::common::protocol::SessionContext;
use crate::features::feature::{Feature, TYPE_ROUTER};
use crate::features::routing::context::RoutingContext;
use crate::features::routing::{Router as CoreRouterTrait, RouterFeature};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DomainStrategy {
    #[default]
    AsIs,
    IpIfNonMatch,
    IpOnDemand,
}

pub struct Router {
    pub domain_strategy: DomainStrategy,
    pub rules: Vec<Rule>,
    pub balancers: HashMap<String, Arc<Balancer>>,
    pub default_tag: Option<String>,
}

impl Router {
    pub fn new(rules: Vec<Rule>, default_tag: Option<String>) -> Self {
        Self {
            domain_strategy: DomainStrategy::AsIs,
            rules,
            balancers: HashMap::new(),
            default_tag,
        }
    }

    pub fn with_balancers(
        rules: Vec<Rule>,
        balancers: HashMap<String, Arc<Balancer>>,
        default_tag: Option<String>,
    ) -> Self {
        Self {
            domain_strategy: DomainStrategy::AsIs,
            rules,
            balancers,
            default_tag,
        }
    }

    pub fn set_domain_strategy(&mut self, strategy: DomainStrategy) {
        self.domain_strategy = strategy;
    }

    pub fn add_balancer(&mut self, tag: impl Into<String>, balancer: Arc<Balancer>) {
        self.balancers.insert(tag.into(), balancer);
    }
}

impl Feature for Router {
    fn feature_type(&self) -> &'static str {
        TYPE_ROUTER
    }
}

impl RouterFeature for Router {
    fn pick_outbound(&self, session: &SessionContext) -> Option<&str> {
        for rule in &self.rules {
            if rule.matches(session) {
                if let Some(btag) = &rule.balancer_tag
                    && let Some(balancer) = self.balancers.get(btag)
                    && let Some(picked) = balancer.pick()
                {
                    for candidate in &balancer.candidates {
                        if candidate == &picked {
                            return Some(candidate.as_str());
                        }
                    }
                    for selector in &balancer.selectors {
                        if selector == &picked {
                            return Some(selector.as_str());
                        }
                    }
                }
                if !rule.tag.is_empty() {
                    return Some(&rule.tag);
                }
            }
        }
        self.default_tag.as_deref()
    }
}

#[async_trait]
impl CoreRouterTrait for Router {
    async fn pick_route(&self, ctx: &dyn RoutingContext) -> Result<String> {
        let in_tag = ctx.get_inbound_tag();
        let target_port = ctx.get_target_port();
        let target_domain = ctx.get_target_domain();
        let target_ips = ctx.get_target_ips();
        let net = ctx.get_network();
        let proto = ctx.get_protocol();
        let user = ctx.get_user();
        let src_ips = ctx.get_source_ips();
        let src_port = ctx.get_source_port();
        let src_addr = if !src_ips.is_empty() && src_port > 0 {
            Some(std::net::SocketAddr::new(src_ips[0], src_port))
        } else {
            None
        };
        let is_tcp = net == crate::common::net::Network::Tcp;

        for rule in &self.rules {
            // Check process
            if !rule.process.is_empty() && !rule.matches_process(src_addr, is_tcp) {
                continue;
            }

            // Check inbound
            if !rule.inbound_tags.is_empty() && !rule.inbound_tags.iter().any(|t| t == in_tag) {
                continue;
            }

            // Check network
            if let Some(r_net) = rule.network
                && r_net != net
            {
                continue;
            }

            // Check ports
            if !rule.ports.is_empty() || !rule.port_ranges.is_empty() {
                let port_match = rule.ports.contains(&target_port)
                    || rule
                        .port_ranges
                        .iter()
                        .any(|&(s, e)| target_port >= s && target_port <= e);
                if !port_match {
                    continue;
                }
            }

            // Check source ports
            if !rule.source_ports.is_empty() && !rule.source_ports.contains(&src_port) {
                continue;
            }

            // Check source IPs
            if !rule.source_ip_matchers.is_empty() {
                let matched = src_ips
                    .iter()
                    .any(|ip| rule.source_ip_matchers.iter().any(|m| m.matches(ip)));
                if !matched {
                    continue;
                }
            }

            // Check protocol
            if !rule.protocols.is_empty() {
                let matched = match proto {
                    Some(p) => rule.protocols.iter().any(|rp| rp.eq_ignore_ascii_case(p)),
                    None => false,
                };
                if !matched {
                    continue;
                }
            }

            // Check user
            if !rule.user_emails.is_empty() {
                let matched = match user {
                    Some(u) => rule.user_emails.iter().any(|ru| ru == u),
                    None => false,
                };
                if !matched {
                    continue;
                }
            }

            // Check domain and IP
            let has_domain_rules = !rule.domain_matchers.is_empty();
            let has_ip_rules = !rule.ip_matchers.is_empty();

            if has_domain_rules || has_ip_rules {
                let mut matched = false;

                if has_domain_rules && let Some(domain) = target_domain {
                    for matcher in &rule.domain_matchers {
                        if matcher.matches(domain) {
                            matched = true;
                            break;
                        }
                    }
                }

                if !matched && has_ip_rules {
                    for ip in &target_ips {
                        if rule.matches_destination_ip(ip) {
                            matched = true;
                            break;
                        }
                    }
                }

                if !matched {
                    continue;
                }
            }

            // Rule matched!
            if let Some(btag) = &rule.balancer_tag
                && let Some(balancer) = self.balancers.get(btag)
                && let Some(picked) = balancer.pick()
            {
                return Ok(picked);
            }

            if !rule.tag.is_empty() {
                return Ok(rule.tag.clone());
            }
        }

        self.default_tag
            .clone()
            .ok_or_else(|| Error::NotFound("No matching route found".into()))
    }
}
