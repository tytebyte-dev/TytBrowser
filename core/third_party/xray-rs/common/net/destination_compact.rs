// Module: common\net\destination_compact.rs
use super::Destination;

pub fn to_compact_string(dest: &Destination) -> String {
    format!("{}:{}", dest.address, dest.port)
}
