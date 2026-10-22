// Module: common\strmatcher\mph_matcher.rs
// 1:1 Rust implementation corresponding to Go common\strmatcher\mph_matcher.go

use super::ac_automaton_matcher::AcAutomatonMatcher;
use super::matchers::RegexMatcher;
use super::strmatcher::MatcherType;
use crate::common::errors::Result;
use std::collections::HashMap;

pub const PRIME_RK: u32 = 16777619;

pub fn rolling_hash(s: &str) -> u32 {
    let mut h = 0u32;
    for &b in s.as_bytes().iter().rev() {
        h = h.wrapping_mul(PRIME_RK).wrapping_add(b as u32);
    }
    h
}

#[derive(Default)]
pub struct MphMatcherGroup {
    ac: Option<AcAutomatonMatcher>,
    other_matchers: Vec<(RegexMatcher, u32)>,
    rule_map: HashMap<String, u32>,
    rules: Vec<String>,
    count: u32,
}

impl MphMatcherGroup {
    pub fn new() -> Self {
        Self {
            ac: None,
            other_matchers: Vec::new(),
            rule_map: HashMap::new(),
            rules: Vec::new(),
            count: 1,
        }
    }

    pub fn add_full_or_domain_pattern(&mut self, pattern: &str, t: MatcherType) {
        let h = rolling_hash(pattern);
        match t {
            MatcherType::Domain => {
                let dot_pattern = format!(".{}", pattern);
                let dot_hash = h.wrapping_mul(PRIME_RK).wrapping_add(b'.' as u32);
                self.rule_map.insert(dot_pattern, dot_hash);
                self.rule_map.insert(pattern.to_string(), h);
            }
            MatcherType::Full => {
                self.rule_map.insert(pattern.to_string(), h);
            }
            _ => {}
        }
    }

    pub fn add_pattern(&mut self, pattern: &str, t: MatcherType) -> Result<u32> {
        match t {
            MatcherType::Substr => {
                let ac = self.ac.get_or_insert_with(AcAutomatonMatcher::new);
                ac.add(pattern, t);
            }
            MatcherType::Full | MatcherType::Domain => {
                let pattern_lower = pattern.to_lowercase();
                self.add_full_or_domain_pattern(&pattern_lower, t);
            }
            MatcherType::Regex => {
                let r = RegexMatcher::new(pattern)?;
                self.other_matchers.push((r, self.count));
            }
        }
        Ok(self.count)
    }

    pub fn build(&mut self) {
        if let Some(ac) = &mut self.ac {
            ac.build();
        }
        self.rules = self.rule_map.keys().cloned().collect();
    }

    pub fn lookup(&self, s: &str) -> bool {
        self.rule_map.contains_key(s)
    }

    pub fn match_pattern(&self, pattern: &str) -> Vec<u32> {
        let bytes = pattern.as_bytes();
        for (i, &b) in bytes.iter().enumerate() {
            if b == b'.' && self.lookup(&pattern[i..]) {
                return vec![1];
            }
        }
        if self.lookup(pattern) {
            return vec![1];
        }
        if let Some(ac) = &self.ac
            && ac.match_str(pattern)
        {
            return vec![1];
        }
        for (m, id) in &self.other_matchers {
            if super::strmatcher::Matcher::match_str(m, pattern) {
                return vec![*id];
            }
        }
        Vec::new()
    }

    pub fn size(&self) -> u32 {
        self.count
    }
}
