// Module: common\drain\drainer.rs
// 1:1 Rust implementation corresponding to Go common\drain\drainer.go

use std::io;
use tokio::io::{AsyncRead, AsyncReadExt};

use super::drain::{Drainer, drain_read_n};
use crate::common::dice::{DeterministicDice, roll};
use crate::common::errors::{Error, Result};

pub struct BehaviorSeedLimitedDrainer {
    pub drain_size: usize,
}

impl BehaviorSeedLimitedDrainer {
    pub fn new(
        behavior_seed: i64,
        drain_foundation: usize,
        max_base_drain_size: usize,
        max_rand_drain: usize,
    ) -> Self {
        let mut behavior_rand = DeterministicDice::new(behavior_seed);
        let base_drain_size = behavior_rand.roll(max_base_drain_size);
        let rand_drain_max = behavior_rand.roll(max_rand_drain) + 1;
        let rand_drain_rolled = roll(rand_drain_max);
        let drain_size = drain_foundation + base_drain_size + rand_drain_rolled;

        Self { drain_size }
    }

    pub fn acknowledge_receive(&mut self, size: usize) {
        if size >= self.drain_size {
            self.drain_size = 0;
        } else {
            self.drain_size -= size;
        }
    }

    pub async fn drain<R: AsyncRead + Unpin>(&self, reader: &mut R) -> Result<()> {
        if self.drain_size > 0 {
            let mut remaining = self.drain_size;
            let mut buf = vec![0u8; remaining.min(8192)];
            while remaining > 0 {
                let to_read = remaining.min(buf.len());
                match reader.read_exact(&mut buf[..to_read]).await {
                    Ok(_) => remaining -= to_read,
                    Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                    Err(e) => return Err(Error::Io(e)),
                }
            }
        }
        Ok(())
    }
}

impl Drainer for BehaviorSeedLimitedDrainer {
    fn acknowledge_receive(&mut self, size: usize) {
        self.acknowledge_receive(size);
    }
}

pub struct NopDrainer;

impl Drainer for NopDrainer {
    fn acknowledge_receive(&mut self, _size: usize) {}
}

impl NopDrainer {
    pub fn new() -> Self {
        Self
    }

    pub async fn drain<R: AsyncRead + Unpin>(&self, _reader: &mut R) -> Result<()> {
        Ok(())
    }
}

impl Default for NopDrainer {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn with_error<D: Drainer, R: AsyncRead + Unpin>(
    drainer: &mut D,
    reader: &mut R,
    err: Error,
) -> Error {
    let _ = drain_read_n(reader, 1024).await;
    drainer.acknowledge_receive(0);
    err
}
