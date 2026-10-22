// Module: app\router\weight.rs
// 1:1 Rust implementation corresponding to Go app\router\weight.go

use regex::Regex;
use std::collections::HashMap;
use std::sync::RwLock;

pub type WeightScaler = fn(value: f64, weight: f64) -> f64;

lazy_static::lazy_static! {
    static ref NUMBER_FINDER: Regex = Regex::new(r"\d+(\.\d+)?").unwrap();
}

#[derive(Debug, Clone)]
pub struct StrategyWeight {
    pub pattern: String,
    pub value: f64,
    pub is_regexp: bool,
}

impl StrategyWeight {
    pub fn new(pattern: impl Into<String>, value: f64) -> Self {
        Self {
            pattern: pattern.into(),
            value,
            is_regexp: false,
        }
    }

    pub fn regex(pattern: impl Into<String>, value: f64) -> Self {
        Self {
            pattern: pattern.into(),
            value,
            is_regexp: true,
        }
    }
}

pub struct WeightManager {
    weights: Vec<StrategyWeight>,
    default_weight: f64,
    scaler: WeightScaler,
    cache: RwLock<HashMap<String, f64>>,
}

impl WeightManager {
    pub fn new(weights: Vec<StrategyWeight>, default_weight: f64, scaler: WeightScaler) -> Self {
        Self {
            weights,
            default_weight,
            scaler,
            cache: RwLock::new(HashMap::new()),
        }
    }

    pub fn get(&self, tag: &str) -> f64 {
        if let Ok(guard) = self.cache.read()
            && let Some(&w) = guard.get(tag)
        {
            return w;
        }

        let w = self.find_value(tag);
        if let Ok(mut guard) = self.cache.write() {
            guard.insert(tag.to_string(), w);
        }
        w
    }

    pub fn apply(&self, tag: &str, value: f64) -> f64 {
        (self.scaler)(value, self.get(tag))
    }

    fn find_value(&self, tag: &str) -> f64 {
        for w in &self.weights {
            let matched = self.get_match(tag, &w.pattern, w.is_regexp);
            if matched.is_empty() {
                continue;
            }
            if w.value > 0.0 {
                return w.value;
            }
            // Auto weight from matched numbers in tag
            if let Some(mat) = NUMBER_FINDER.find(&matched)
                && let Ok(val) = mat.as_str().parse::<f64>()
            {
                return val;
            }
            return self.default_weight;
        }
        self.default_weight
    }

    fn get_match(&self, tag: &str, find: &str, is_regexp: bool) -> String {
        if !is_regexp {
            if tag.contains(find) || find == "*" {
                find.to_string()
            } else {
                String::new()
            }
        } else if let Ok(re) = Regex::new(find) {
            re.find(tag)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default()
        } else {
            String::new()
        }
    }
}
