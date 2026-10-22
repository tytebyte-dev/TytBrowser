// Module: common\strmatcher\matchers.rs
// 1:1 Rust implementation corresponding to Go common\strmatcher\matchers.go

use super::strmatcher::{Matcher, MatcherType};
use crate::common::errors::{Error, Result};
use regex::Regex;

pub struct SubstrMatcher {
    pattern: String,
}

impl SubstrMatcher {
    pub fn new(pattern: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
        }
    }
}

impl Matcher for SubstrMatcher {
    fn match_str(&self, input: &str) -> bool {
        input.contains(&self.pattern)
    }

    fn pattern(&self) -> &str {
        &self.pattern
    }

    fn matcher_type(&self) -> MatcherType {
        MatcherType::Substr
    }
}

pub struct RegexMatcher {
    pattern: String,
    regex: Regex,
}

impl RegexMatcher {
    pub fn new(pattern: impl Into<String>) -> Result<Self> {
        let p = pattern.into();
        let regex = Regex::new(&p).map_err(|e| Error::Config(format!("Invalid regex: {}", e)))?;
        Ok(Self { pattern: p, regex })
    }
}

impl Matcher for RegexMatcher {
    fn match_str(&self, input: &str) -> bool {
        self.regex.is_match(input)
    }

    fn pattern(&self) -> &str {
        &self.pattern
    }

    fn matcher_type(&self) -> MatcherType {
        MatcherType::Regex
    }
}
