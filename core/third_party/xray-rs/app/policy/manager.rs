// Module: app\policy\manager.rs
// 1:1 Rust implementation corresponding to Go app\policy\manager.go

use std::collections::HashMap;
use std::sync::RwLock;

use super::config::default_policy;
use super::config_pb::{Config, Policy, SystemPolicy};
use crate::common::errors::Result;
use crate::features::feature::{Feature, TYPE_POLICY_MANAGER};
use crate::features::policy::{
    PolicyManager as IPolicyManager, SessionPolicy, SystemPolicy as CoreSystemPolicy,
    session_default,
};

/// Instance is an instance of Policy manager.
pub struct Instance {
    levels: HashMap<u32, Policy>,
    system: Option<SystemPolicy>,
}

impl Instance {
    pub fn new(config: &Config) -> Result<Self> {
        let mut levels = HashMap::new();
        for (&lv, p) in &config.level {
            let mut pp = default_policy();
            pp.override_with(p);
            levels.insert(lv, pp);
        }

        Ok(Self {
            levels,
            system: config.system.clone(),
        })
    }

    pub fn for_level(&self, level: u32) -> SessionPolicy {
        if let Some(p) = self.levels.get(&level) {
            return p.to_core_policy();
        }
        session_default()
    }

    pub fn for_system(&self) -> CoreSystemPolicy {
        match &self.system {
            Some(s) => s.to_core_policy(),
            None => CoreSystemPolicy::default(),
        }
    }
}

impl Feature for Instance {
    fn feature_type(&self) -> &'static str {
        TYPE_POLICY_MANAGER
    }

    fn start(&self) -> Result<()> {
        Ok(())
    }

    fn close(&self) -> Result<()> {
        Ok(())
    }
}

impl IPolicyManager for Instance {
    fn for_level(&self, level: u32) -> SessionPolicy {
        self.for_level(level)
    }

    fn for_system(&self) -> CoreSystemPolicy {
        self.for_system()
    }
}

// Legacy struct for backward compatibility
pub struct ConfigPolicyManager {
    levels: RwLock<HashMap<u32, super::config::PolicyLevelConfig>>,
}

impl ConfigPolicyManager {
    pub fn new() -> Self {
        Self {
            levels: RwLock::new(HashMap::new()),
        }
    }

    pub fn set_level_policy(&self, level: u32, policy: super::config::PolicyLevelConfig) {
        let mut guard = self.levels.write().unwrap();
        guard.insert(level, policy);
    }

    pub fn for_level(&self, level: u32) -> super::config::PolicyLevelConfig {
        let guard = self.levels.read().unwrap();
        guard.get(&level).cloned().unwrap_or_default()
    }
}

impl Default for ConfigPolicyManager {
    fn default() -> Self {
        Self::new()
    }
}
