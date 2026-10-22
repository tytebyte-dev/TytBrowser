// Module: transport\internet\grpc\encoding\stream.pb.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hunk {
    pub data: Vec<u8>,
}
