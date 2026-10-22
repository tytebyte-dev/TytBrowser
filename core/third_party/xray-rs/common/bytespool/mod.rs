use std::sync::Mutex;

const NUM_POOLS: usize = 4;
const POOL_SIZES: [usize; NUM_POOLS] = [2048, 8192, 32768, 131072];

pub struct BytesPool {
    pools: [Mutex<Vec<Vec<u8>>>; NUM_POOLS],
}

impl BytesPool {
    pub const fn new() -> Self {
        Self {
            pools: [
                Mutex::new(Vec::new()),
                Mutex::new(Vec::new()),
                Mutex::new(Vec::new()),
                Mutex::new(Vec::new()),
            ],
        }
    }

    pub fn alloc(&self, size: usize) -> Vec<u8> {
        for (idx, &pool_size) in POOL_SIZES.iter().enumerate() {
            if size <= pool_size {
                if let Ok(mut guard) = self.pools[idx].lock()
                    && let Some(mut buf) = guard.pop()
                {
                    buf.resize(size, 0);
                    return buf;
                }
                return vec![0u8; size];
            }
        }
        vec![0u8; size]
    }

    pub fn free(&self, mut buf: Vec<u8>) {
        let cap = buf.capacity();
        for (idx, &pool_size) in POOL_SIZES.iter().enumerate() {
            if cap >= pool_size && (idx == NUM_POOLS - 1 || cap < POOL_SIZES[idx + 1]) {
                buf.clear();
                if let Ok(mut guard) = self.pools[idx].lock()
                    && guard.len() < 128
                {
                    guard.push(buf);
                }
                return;
            }
        }
    }
}

impl Default for BytesPool {
    fn default() -> Self {
        Self::new()
    }
}

pub static GLOBAL_POOL: BytesPool = BytesPool::new();

pub fn alloc(size: usize) -> Vec<u8> {
    GLOBAL_POOL.alloc(size)
}

pub fn free(buf: Vec<u8>) {
    GLOBAL_POOL.free(buf)
}
