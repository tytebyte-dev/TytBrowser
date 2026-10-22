// Module: proxy\vless\encoding\addons.pb.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Addons {
    pub flow: String,
    pub seed: Vec<u8>,
}
