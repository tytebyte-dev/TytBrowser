pub mod latency;
pub mod peer;

#[cfg(test)]
pub mod peer_test;

pub use latency::{AverageLatency, HasLatency, Latency};
pub use peer::Peer;
