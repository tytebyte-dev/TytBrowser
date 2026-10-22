// Module: common\net\address.pb.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IPOrDomain {
    Ip(Vec<u8>),
    Domain(String),
}

impl Default for IPOrDomain {
    fn default() -> Self {
        Self::Domain(String::new())
    }
}
