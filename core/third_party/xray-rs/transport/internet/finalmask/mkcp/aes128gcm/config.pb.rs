// Module: transport\internet\finalmask\mkcp\aes128gcm\config.pb.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub key: String,
}
