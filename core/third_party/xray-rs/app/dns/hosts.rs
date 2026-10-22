use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::RwLock;

pub struct StaticHosts {
    exact: RwLock<HashMap<String, Vec<IpAddr>>>,
    domain_suffix: RwLock<Vec<(String, Vec<IpAddr>)>>,
    keywords: RwLock<Vec<(String, Vec<IpAddr>)>>,
}

impl StaticHosts {
    pub fn new() -> Self {
        Self {
            exact: RwLock::new(HashMap::new()),
            domain_suffix: RwLock::new(Vec::new()),
            keywords: RwLock::new(Vec::new()),
        }
    }

    pub fn add_exact(&self, domain: &str, ips: Vec<IpAddr>) {
        if let Ok(mut guard) = self.exact.write() {
            guard.insert(domain.to_lowercase(), ips);
        }
    }

    pub fn add_domain_suffix(&self, suffix: &str, ips: Vec<IpAddr>) {
        if let Ok(mut guard) = self.domain_suffix.write() {
            guard.push((suffix.to_lowercase(), ips));
        }
    }

    pub fn add_keyword(&self, keyword: &str, ips: Vec<IpAddr>) {
        if let Ok(mut guard) = self.keywords.write() {
            guard.push((keyword.to_lowercase(), ips));
        }
    }

    pub fn lookup(&self, domain: &str) -> Option<Vec<IpAddr>> {
        let domain_lower = domain.to_lowercase();

        // 1. Exact match
        if let Ok(guard) = self.exact.read()
            && let Some(ips) = guard.get(&domain_lower)
        {
            return Some(ips.clone());
        }

        // 2. Domain suffix match (e.g. "google.com" matches "mail.google.com" or "google.com")
        if let Ok(guard) = self.domain_suffix.read() {
            for (suffix, ips) in guard.iter() {
                if domain_lower == *suffix || domain_lower.ends_with(&format!(".{}", suffix)) {
                    return Some(ips.clone());
                }
            }
        }

        // 3. Keyword match
        if let Ok(guard) = self.keywords.read() {
            for (keyword, ips) in guard.iter() {
                if domain_lower.contains(keyword) {
                    return Some(ips.clone());
                }
            }
        }

        None
    }
}

impl Default for StaticHosts {
    fn default() -> Self {
        Self::new()
    }
}
