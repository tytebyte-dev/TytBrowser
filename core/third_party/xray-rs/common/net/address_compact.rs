// Module: common\net\address_compact.rs
use super::Address;

pub fn is_loopback(addr: &Address) -> bool {
    addr.is_loopback()
}
