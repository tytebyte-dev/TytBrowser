// Module: common\strmatcher\strmatcher.rs
// 1:1 Rust implementation corresponding to Go common\strmatcher\strmatcher.go

use super::domain_matcher::{DomainMatcher, DomainMatcherGroup};
use super::full_matcher::{FullMatcher, FullMatcherGroup};
use super::matchers::{RegexMatcher, SubstrMatcher};
use crate::common::errors::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MatcherType {
    Full = 0,
    Substr = 1,
    Domain = 2,
    Regex = 3,
}

impl MatcherType {
    pub fn new_matcher(&self, pattern: &str) -> Result<Box<dyn Matcher>> {
        match self {
            MatcherType::Full => Ok(Box::new(FullMatcher::new(pattern))),
            MatcherType::Substr => Ok(Box::new(SubstrMatcher::new(pattern))),
            MatcherType::Domain => Ok(Box::new(DomainMatcher::new(pattern))),
            MatcherType::Regex => Ok(Box::new(RegexMatcher::new(pattern)?)),
        }
    }
}

pub trait Matcher: Send + Sync {
    fn match_str(&self, input: &str) -> bool;
    fn pattern(&self) -> &str;
    fn matcher_type(&self) -> MatcherType;
}

pub trait IndexMatcher: Send + Sync {
    fn match_index(&self, input: &str) -> Vec<u32>;
    fn size(&self) -> u32;
}

pub struct MatcherEntry {
    pub m: Box<dyn Matcher>,
    pub id: u32,
}

#[derive(Default)]
pub struct MatcherGroup {
    count: u32,
    full_matcher: FullMatcherGroup,
    domain_matcher: DomainMatcherGroup,
    other_matchers: Vec<MatcherEntry>,
}

impl MatcherGroup {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, m: Box<dyn Matcher>) -> u32 {
        self.count += 1;
        let c = self.count;
        match m.matcher_type() {
            MatcherType::Full => {
                self.full_matcher.add(m.pattern(), c);
            }
            MatcherType::Domain => {
                self.domain_matcher.add(m.pattern(), c);
            }
            _ => {
                self.other_matchers.push(MatcherEntry { m, id: c });
            }
        }
        c
    }

    pub fn match_pattern(&self, pattern: &str) -> Vec<u32> {
        let mut result = Vec::new();
        result.extend(self.full_matcher.match_str(pattern));
        result.extend(self.domain_matcher.match_domain(pattern));
        for e in &self.other_matchers {
            if e.m.match_str(pattern) {
                result.push(e.id);
            }
        }
        result
    }

    pub fn match_any(&self, pattern: &str) -> bool {
        !self.match_pattern(pattern).is_empty()
    }

    pub fn match_all_indices(&self, pattern: &str) -> Vec<usize> {
        self.match_pattern(pattern)
            .into_iter()
            .map(|id| (id.saturating_sub(1)) as usize)
            .collect()
    }

    pub fn size(&self) -> u32 {
        self.count
    }
}

impl IndexMatcher for MatcherGroup {
    fn match_index(&self, input: &str) -> Vec<u32> {
        self.match_pattern(input)
    }

    fn size(&self) -> u32 {
        self.size()
    }
}

pub struct IndexMatcherGroup {
    pub matchers: Vec<Box<dyn IndexMatcher>>,
}

impl IndexMatcherGroup {
    pub fn new() -> Self {
        Self {
            matchers: Vec::new(),
        }
    }

    pub fn match_input(&self, input: &str) -> Vec<u32> {
        let mut offset = 0;
        for m in &self.matchers {
            let res = m.match_index(input);
            if !res.is_empty() {
                if offset == 0 {
                    return res;
                }
                return res.into_iter().map(|id| id + offset).collect();
            }
            offset += m.size();
        }
        Vec::new()
    }

    pub fn size(&self) -> u32 {
        self.matchers.iter().map(|m| m.size()).sum()
    }
}

impl Default for IndexMatcherGroup {
    fn default() -> Self {
        Self::new()
    }
}
