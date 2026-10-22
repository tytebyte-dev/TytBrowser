pub mod serial;
pub mod string;
pub mod typed_message;
#[path = "typed_message.pb.rs"]
pub mod typed_message_pb;

#[cfg(test)]
pub mod serial_test;
#[cfg(test)]
pub mod string_test;
#[cfg(test)]
pub mod typed_message_test;

pub use serial::{
    read_u16, read_u32, read_u64, read_uint16, write_u16, write_u32, write_u64, write_uint16,
    write_uint64,
};
pub use string::{StringValue, concat, concat_strings, to_string, to_string_val};
pub use typed_message::{TypedMessage, get_instance, to_typed_message};
