pub mod ac_automaton_matcher;
pub mod domain_matcher;
pub mod full_matcher;
pub mod matchers;
pub mod mph_matcher;
#[path = "mph_matcher_compact.rs"]
pub mod mph_matcher_compact;
pub mod strmatcher;

#[cfg(test)]
pub mod benchmark_test;
#[cfg(test)]
pub mod domain_matcher_test;
#[cfg(test)]
pub mod full_matcher_test;
#[cfg(test)]
pub mod matchers_test;
#[cfg(test)]
pub mod strmatcher_test;

pub use ac_automaton_matcher::AcAutomatonMatcher;
pub use domain_matcher::{DomainMatcher, DomainMatcherGroup};
pub use full_matcher::{FullMatcher, FullMatcherGroup};
pub use matchers::{RegexMatcher, SubstrMatcher};
pub use mph_matcher::{MphMatcherGroup, PRIME_RK, rolling_hash};
pub use mph_matcher_compact::CompactMatcher;
pub use strmatcher::{
    IndexMatcher, IndexMatcherGroup, Matcher, MatcherEntry, MatcherGroup, MatcherType,
};
