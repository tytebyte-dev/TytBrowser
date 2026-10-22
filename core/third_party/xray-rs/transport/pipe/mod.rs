// Module: transport\pipe\mod.rs
// 1:1 Rust module definition for pipe package

#[path = "impl.rs"]
pub mod impl_;
pub mod pipe;
pub mod reader;
pub mod writer;

#[cfg(test)]
pub mod pipe_test;

pub use self::impl_::{Pipe, PipeOption, State};
pub use self::pipe::{
    OptionFn, discard_overflow, new_pipe, new_with_options, with_size_limit, without_size_limit,
};
pub use self::reader::Reader;
pub use self::writer::Writer;

pub type PipeReader = Reader;
pub type PipeWriter = Writer;
