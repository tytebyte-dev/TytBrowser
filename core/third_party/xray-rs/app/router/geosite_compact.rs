use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct GeoSiteCompactList {
    pub sites: HashMap<String, Vec<String>>,
    pub deps: HashMap<String, Vec<String>>,
}

impl GeoSiteCompactList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_site(&mut self, country_code: impl Into<String>, domains: Vec<String>) {
        self.sites
            .insert(country_code.into().to_uppercase(), domains);
    }

    pub fn add_dep(&mut self, country_code: impl Into<String>, dep: impl Into<String>) {
        self.deps
            .entry(country_code.into().to_uppercase())
            .or_default()
            .push(dep.into().to_uppercase());
    }

    pub fn get_all_domains(&self, country_code: &str) -> Vec<String> {
        let mut result = Vec::new();
        let code = country_code.to_uppercase();
        if let Some(list) = self.sites.get(&code) {
            result.extend(list.clone());
        }

        if let Some(deps) = self.deps.get(&code) {
            for dep in deps {
                if let Some(dep_list) = self.sites.get(dep) {
                    result.extend(dep_list.clone());
                }
            }
        }
        result
    }
}
