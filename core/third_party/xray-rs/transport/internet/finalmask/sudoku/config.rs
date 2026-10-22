// Module: transport\internet\finalmask\sudoku\config.rs
// 1:1 Rust implementation corresponding to Go transport\internet\finalmask\sudoku\config.go

use crate::common::errors::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SudokuConfig {
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub ascii: String,
    #[serde(default)]
    pub custom_table: String,
    #[serde(default)]
    pub padding_min: u32,
    #[serde(default)]
    pub padding_max: u32,
    #[serde(default)]
    pub custom_tables: Vec<String>,
}

impl SudokuConfig {
    pub fn new(password: impl Into<String>) -> Self {
        Self {
            password: password.into(),
            ascii: String::new(),
            custom_table: String::new(),
            padding_min: 0,
            padding_max: 0,
            custom_tables: Vec::new(),
        }
    }

    pub fn normalized_ascii(&self) -> Result<String> {
        let mode = self.ascii.trim().to_ascii_lowercase();
        match mode.as_str() {
            "" | "entropy" | "prefer_entropy" => Ok("prefer_entropy".to_string()),
            "ascii" | "prefer_ascii" => Ok("prefer_ascii".to_string()),
            _ => Err(Error::Config(format!(
                "invalid sudoku ascii mode: {}",
                self.ascii
            ))),
        }
    }

    pub fn normalized_custom_patterns(&self, mode: &str) -> Result<Vec<String>> {
        if mode == "prefer_ascii" {
            return Ok(vec![String::new()]);
        }

        let mut raw_patterns = self.custom_tables.clone();
        if raw_patterns.is_empty() && !self.custom_table.is_empty() {
            raw_patterns.push(self.custom_table.clone());
        }

        let mut patterns = Vec::new();
        let mut seen = HashSet::new();

        for raw in raw_patterns {
            let pattern = raw.trim();
            let norm = if !pattern.is_empty() {
                Self::normalize_custom_table(pattern)?
            } else {
                String::new()
            };

            if seen.insert(norm.clone()) {
                patterns.push(norm);
            }
        }

        if patterns.is_empty() {
            patterns.push(String::new());
        }

        Ok(patterns)
    }

    pub fn normalize_custom_table(pattern: &str) -> Result<String> {
        let cleaned: String = pattern
            .trim()
            .to_ascii_lowercase()
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        if cleaned.len() != 8 {
            return Err(Error::Config(format!(
                "customTable must be 8 chars, got {}",
                cleaned.len()
            )));
        }

        let mut x_count = 0;
        let mut p_count = 0;
        let mut v_count = 0;

        for ch in cleaned.chars() {
            match ch {
                'x' => x_count += 1,
                'p' => p_count += 1,
                'v' => v_count += 1,
                _ => {
                    return Err(Error::Config(format!(
                        "customTable has invalid char {:?}",
                        ch
                    )));
                }
            }
        }

        if x_count != 2 || p_count != 2 || v_count != 4 {
            return Err(Error::Config(
                "customTable must contain exactly 2 x, 2 p and 4 v".to_string(),
            ));
        }

        Ok(cleaned)
    }

    pub fn normalized_padding(&self) -> (usize, usize) {
        let p_min = self.padding_min.min(100) as usize;
        let mut p_max = self.padding_max.min(100) as usize;
        if p_max < p_min {
            p_max = p_min;
        }
        (p_min, p_max)
    }
}
