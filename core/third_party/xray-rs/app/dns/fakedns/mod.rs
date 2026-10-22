pub mod fake;
pub mod fakedns;

#[cfg(test)]
pub mod fakedns_test;

pub use fake::FakeDnsHolder;
pub use fakedns::FakeDnsConfig;
