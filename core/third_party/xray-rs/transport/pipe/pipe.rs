// Module: transport\pipe\pipe.rs
// 1:1 Rust implementation corresponding to Go transport\pipe\pipe.go

use super::impl_::{Pipe, PipeOption};
use super::reader::Reader;
use super::writer::Writer;

pub type OptionFn = Box<dyn FnOnce(&mut PipeOption)>;

pub fn without_size_limit() -> OptionFn {
    Box::new(|opt: &mut PipeOption| {
        opt.limit = -1;
    })
}

pub fn with_size_limit(limit: i32) -> OptionFn {
    Box::new(move |opt: &mut PipeOption| {
        opt.limit = limit;
    })
}

pub fn discard_overflow() -> OptionFn {
    Box::new(|opt: &mut PipeOption| {
        opt.discard_overflow = true;
    })
}

pub fn new_with_options(opts: Vec<OptionFn>) -> (Reader, Writer) {
    let mut opt = PipeOption::new();
    for o in opts {
        o(&mut opt);
    }
    new_pipe(opt)
}

pub fn new_pipe(opt: PipeOption) -> (Reader, Writer) {
    let p = Pipe::new(opt);
    (Reader::new(p.clone()), Writer::new(p))
}
