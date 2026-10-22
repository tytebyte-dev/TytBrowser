// Module: common\mux\client.rs
// 1:1 Rust implementation corresponding to Go common\mux\client.go

use super::frame::Frame;
use super::session::{Session, SessionManager};
use crate::common::errors::Result;
use crate::common::net::Destination;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct MuxClient {
    session_manager: Arc<SessionManager>,
}

impl MuxClient {
    pub fn new() -> Self {
        Self {
            session_manager: Arc::new(SessionManager::new()),
        }
    }

    pub fn active_sessions(&self) -> usize {
        self.session_manager.size()
    }

    pub async fn new_session(
        &self,
        target: Destination,
        sender: mpsc::Sender<Vec<u8>>,
    ) -> Result<(Session, Frame)> {
        let session = self
            .session_manager
            .allocate_channel(target.clone(), sender)
            .await?;
        let frame = Frame::new_session(session.id, target, Vec::new());
        Ok((session, frame))
    }

    pub async fn close_session(&self, id: u16) -> Option<Frame> {
        if self.session_manager.remove(false, id).is_some() {
            Some(Frame::end(id))
        } else {
            None
        }
    }
}

impl Default for MuxClient {
    fn default() -> Self {
        Self::new()
    }
}

pub type Client = MuxClient;
