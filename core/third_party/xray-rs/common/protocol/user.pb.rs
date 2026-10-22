// Module: common\protocol\user.pb.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub level: u32,
    pub email: String,
}
