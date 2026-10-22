use crate::app::stats::Counter;
use std::sync::Arc;

pub struct SizeStatCounter {
    pub read_counter: Option<Arc<Counter>>,
    pub write_counter: Option<Arc<Counter>>,
}

impl SizeStatCounter {
    pub fn new(read_counter: Option<Arc<Counter>>, write_counter: Option<Arc<Counter>>) -> Self {
        Self {
            read_counter,
            write_counter,
        }
    }

    pub fn record_read(&self, n: usize) {
        if let Some(c) = &self.read_counter {
            c.add(n as i64);
        }
    }

    pub fn record_write(&self, n: usize) {
        if let Some(c) = &self.write_counter {
            c.add(n as i64);
        }
    }
}
