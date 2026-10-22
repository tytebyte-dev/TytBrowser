// Module: common\mux\session.rs
// 1:1 Rust implementation corresponding to Go common\mux\session.go

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

use crate::common::errors::{Error, Result};
use crate::common::mux::mux::ClientStrategy;
use crate::common::net::Destination;
use crate::common::signal::done::Done;

#[derive(Clone)]
pub struct Session {
    pub id: u16,
    pub target: Option<Destination>,
    pub sender: Option<mpsc::Sender<Vec<u8>>>,
    pub closed: bool,
    pub done: Arc<Done>,
}

impl Session {
    pub fn new(id: u16) -> Self {
        Self {
            id,
            target: None,
            sender: None,
            closed: false,
            done: Arc::new(Done::new()),
        }
    }

    pub fn close(&mut self) {
        if self.closed {
            return;
        }
        self.closed = true;
        self.done.close();
    }
}

pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<u16, Session>>>,
    count: Arc<AtomicU16>,
    closed: Arc<AtomicBool>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::with_capacity(16))),
            count: Arc::new(AtomicU16::new(0)),
            closed: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn closed(&self) -> bool {
        self.closed.load(Ordering::Relaxed)
    }

    pub fn is_closed(&self) -> bool {
        self.closed()
    }

    pub fn size(&self) -> usize {
        let guard = self.sessions.read().unwrap();
        guard.len()
    }

    pub fn count(&self) -> usize {
        self.count.load(Ordering::Relaxed) as usize
    }

    pub fn allocate(&self, strategy: &ClientStrategy) -> Option<Session> {
        let max_concurrency = strategy.max_concurrency as usize;
        let max_connection = strategy.max_connection;

        let mut guard = self.sessions.write().unwrap();
        if self.closed.load(Ordering::Relaxed) {
            return None;
        }

        if max_concurrency > 0 && guard.len() >= max_concurrency {
            return None;
        }

        let curr_count = self.count.load(Ordering::Relaxed);
        if max_connection > 0 && curr_count >= max_connection as u16 {
            return None;
        }

        let id = self.count.fetch_add(1, Ordering::SeqCst) + 1;
        let session = Session::new(id);
        guard.insert(id, session.clone());
        Some(session)
    }

    pub async fn allocate_channel(
        &self,
        target: Destination,
        sender: mpsc::Sender<Vec<u8>>,
    ) -> Result<Session> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(Error::Closed);
        }

        let id = self.count.fetch_add(1, Ordering::SeqCst) + 1;
        let mut session = Session::new(id);
        session.target = Some(target);
        session.sender = Some(sender);

        let mut guard = self.sessions.write().unwrap();
        guard.insert(id, session.clone());
        Ok(session)
    }

    pub fn add(&self, session: &Session) -> bool {
        let mut guard = self.sessions.write().unwrap();
        if self.closed.load(Ordering::Relaxed) {
            return false;
        }

        self.count.fetch_add(1, Ordering::SeqCst);
        guard.insert(session.id, session.clone());
        true
    }

    pub fn remove(&self, _locked: bool, id: u16) -> Option<Session> {
        let mut guard = self.sessions.write().unwrap();
        if self.closed.load(Ordering::Relaxed) {
            return None;
        }
        guard.remove(&id)
    }

    pub fn get(&self, id: u16) -> Option<Session> {
        let guard = self.sessions.read().unwrap();
        if self.closed.load(Ordering::Relaxed) {
            return None;
        }
        guard.get(&id).cloned()
    }

    pub fn close_if_no_session_and_idle(&self, check_size: usize, check_count: usize) -> bool {
        let mut guard = self.sessions.write().unwrap();
        if self.closed.load(Ordering::Relaxed) {
            return true;
        }

        let curr_count = self.count.load(Ordering::Relaxed) as usize;
        if !guard.is_empty() || check_size != 0 || check_count != curr_count {
            return false;
        }

        self.closed.store(true, Ordering::SeqCst);
        guard.clear();
        true
    }

    pub fn close(&self) {
        if self.closed.swap(true, Ordering::SeqCst) {
            return;
        }

        let mut guard = self.sessions.write().unwrap();
        for (_, mut session) in guard.drain() {
            session.close();
        }
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
