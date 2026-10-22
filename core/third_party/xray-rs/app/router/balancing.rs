use crate::app::observatory::Observatory;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

pub trait BalancingStrategy: Send + Sync {
    fn name(&self) -> &str;
    fn pick_outbound(&self, tags: &[String]) -> Option<String>;
}

pub struct RandomStrategy {
    pub fallback_tag: Option<String>,
    pub observatory: Option<Arc<Observatory>>,
}

impl RandomStrategy {
    pub fn new() -> Self {
        Self {
            fallback_tag: None,
            observatory: None,
        }
    }

    pub fn with_fallback(
        fallback_tag: Option<String>,
        observatory: Option<Arc<Observatory>>,
    ) -> Self {
        Self {
            fallback_tag,
            observatory,
        }
    }
}

impl Default for RandomStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl BalancingStrategy for RandomStrategy {
    fn name(&self) -> &str {
        "random"
    }

    fn pick_outbound(&self, tags: &[String]) -> Option<String> {
        if tags.is_empty() {
            return self.fallback_tag.clone();
        }

        let alive_tags: Vec<String> = if let Some(obs) = &self.observatory {
            tags.iter()
                .filter(|t| {
                    if let Some(status) = obs.get_status(t) {
                        status.alive
                    } else {
                        true // Not found candidates considered alive
                    }
                })
                .cloned()
                .collect()
        } else {
            tags.to_vec()
        };

        if alive_tags.is_empty() {
            self.fallback_tag.clone()
        } else {
            let idx = rand_index(alive_tags.len());
            Some(alive_tags[idx].clone())
        }
    }
}

pub struct RoundRobinStrategy {
    index: AtomicUsize,
    fallback_tag: Option<String>,
}

impl RoundRobinStrategy {
    pub fn new(fallback_tag: Option<String>) -> Self {
        Self {
            index: AtomicUsize::new(0),
            fallback_tag,
        }
    }
}

impl BalancingStrategy for RoundRobinStrategy {
    fn name(&self) -> &str {
        "roundRobin"
    }

    fn pick_outbound(&self, tags: &[String]) -> Option<String> {
        if tags.is_empty() {
            return self.fallback_tag.clone();
        }
        let cur = self.index.fetch_add(1, Ordering::Relaxed);
        let selected = &tags[cur % tags.len()];
        Some(selected.clone())
    }
}

pub struct LeastPingStrategy {
    observatory: Arc<Observatory>,
    fallback_tag: Option<String>,
}

impl LeastPingStrategy {
    pub fn new(observatory: Arc<Observatory>, fallback_tag: Option<String>) -> Self {
        Self {
            observatory,
            fallback_tag,
        }
    }
}

impl BalancingStrategy for LeastPingStrategy {
    fn name(&self) -> &str {
        "leastPing"
    }

    fn pick_outbound(&self, tags: &[String]) -> Option<String> {
        self.observatory
            .select_best_outbound(tags)
            .or_else(|| self.fallback_tag.clone())
    }
}

pub struct LeastLoadStrategy {
    observatory: Arc<Observatory>,
    fallback_tag: Option<String>,
}

impl LeastLoadStrategy {
    pub fn new(observatory: Arc<Observatory>, fallback_tag: Option<String>) -> Self {
        Self {
            observatory,
            fallback_tag,
        }
    }
}

impl BalancingStrategy for LeastLoadStrategy {
    fn name(&self) -> &str {
        "leastLoad"
    }

    fn pick_outbound(&self, tags: &[String]) -> Option<String> {
        self.observatory
            .select_best_outbound(tags)
            .or_else(|| self.fallback_tag.clone())
    }
}

pub struct Balancer {
    pub tag: String,
    pub selectors: Vec<String>,
    pub candidates: Vec<String>,
    pub strategy: Arc<dyn BalancingStrategy>,
    pub fallback_tag: Option<String>,
}

impl Balancer {
    pub fn new(
        tag: impl Into<String>,
        selectors: Vec<String>,
        strategy: Arc<dyn BalancingStrategy>,
        fallback_tag: Option<String>,
    ) -> Self {
        Self {
            tag: tag.into(),
            selectors,
            candidates: Vec::new(),
            strategy,
            fallback_tag,
        }
    }

    pub fn with_candidates(
        tag: impl Into<String>,
        selectors: Vec<String>,
        candidates: Vec<String>,
        strategy: Arc<dyn BalancingStrategy>,
        fallback_tag: Option<String>,
    ) -> Self {
        Self {
            tag: tag.into(),
            selectors,
            candidates,
            strategy,
            fallback_tag,
        }
    }

    pub fn pick(&self) -> Option<String> {
        let pool = if !self.candidates.is_empty() {
            &self.candidates[..]
        } else {
            &self.selectors[..]
        };
        self.pick_outbound(pool)
    }

    pub fn pick_outbound(&self, candidates: &[String]) -> Option<String> {
        let matched: Vec<String> = candidates
            .iter()
            .filter(|c| {
                self.selectors.is_empty()
                    || self.selectors.iter().any(|s| c.starts_with(s) || s == "*")
            })
            .cloned()
            .collect();

        if matched.is_empty() {
            self.fallback_tag.clone()
        } else {
            self.strategy
                .pick_outbound(&matched)
                .or_else(|| self.fallback_tag.clone())
        }
    }
}

fn rand_index(len: usize) -> usize {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as usize;
    nanos % len
}
