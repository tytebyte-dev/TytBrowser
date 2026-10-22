use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct OcspResponse {
    pub raw: Vec<u8>,
    pub valid_until: Instant,
}

pub struct OcspCache {
    cache: Mutex<Option<OcspResponse>>,
}

impl OcspCache {
    pub const fn new() -> Self {
        Self {
            cache: Mutex::new(None),
        }
    }

    pub fn set(&self, raw: Vec<u8>, ttl: Duration) {
        if let Ok(mut guard) = self.cache.lock() {
            *guard = Some(OcspResponse {
                raw,
                valid_until: Instant::now() + ttl,
            });
        }
    }

    pub fn get(&self) -> Option<Vec<u8>> {
        let guard = self.cache.lock().ok()?;
        if let Some(resp) = guard.as_ref()
            && Instant::now() < resp.valid_until
        {
            return Some(resp.raw.clone());
        }
        None
    }
}

impl Default for OcspCache {
    fn default() -> Self {
        Self::new()
    }
}
