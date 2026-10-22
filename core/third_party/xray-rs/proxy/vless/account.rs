// Module: proxy\vless\account.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub flow: String,
}

impl Account {
    pub fn new(id: impl Into<String>, flow: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            flow: flow.into(),
        }
    }
}
