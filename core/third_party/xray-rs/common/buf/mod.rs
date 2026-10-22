pub mod buf;
pub mod buffer;
pub mod copy;
pub mod io;
pub mod multi_buffer;
#[path = "override.rs"]
pub mod override_;
pub mod reader;
pub mod readv;
pub mod readv_posix;
pub mod readv_reader;
pub mod readv_reader_stub;
pub mod readv_unix;
pub mod readv_windows;
pub mod writer;

#[cfg(test)]
pub mod buffer_test;
#[cfg(test)]
pub mod copy_test;
#[cfg(test)]
pub mod io_test;
#[cfg(test)]
pub mod multi_buffer_test;
#[cfg(test)]
pub mod override_test;
#[cfg(test)]
pub mod reader_test;
#[cfg(test)]
pub mod readv_test;
#[cfg(test)]
pub mod writer_test;

pub use buffer::Buffer;
pub use copy::{CopyOptions, copy, copy_once_timeout, copy_stream};
pub use io::{Reader, TimeoutReader, Writer, write_all_bytes};
pub use multi_buffer::MultiBuffer;
pub use override_::{EndpointOverrideReader, EndpointOverrideWriter};
pub use reader::{BufferedReader, PacketReader, SingleReader, read_buffer};
pub use readv::VectorReader;
pub use readv_reader::{AllocStrategy, ReadVReader};
pub use writer::{BufferedWriter, Discard, SequentialWriter};
