// Module: common\strmatcher\full_matcher.rs
// 1:1 Rust implementation corresponding to Go common\strmatcher\full_matcher.go

use super::strmatcher::{Matcher, MatcherType};
use std::collections::HashMap;

pub struct FullMatcher {
    pattern: String,
}

impl FullMatcher {
    pub fn new(pattern: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
        }
    }
}

impl Matcher for FullMatcher {
    fn match_str(&self, input: &str) -> bool {
        self.pattern == input
    }

    fn pattern(&self) -> &str {
        &self.pattern
    }

    fn matcher_type(&self) -> MatcherType {
        MatcherType::Full
    }
}

#[derive(Default)]
pub struct FullMatcherGroup {
    matchers: HashMap<String, Vec<u32>>,
}

impl FullMatcherGroup {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, domain: &str, value: u32) {
        self.matchers
            .entry(domain.to_string())
            .or_default()
            .push(value);
    }

    pub fn match_str(&self, str: &str) -> Vec<u32> {
        self.matchers.get(str).cloned().unwrap_or_default()
    }
}
