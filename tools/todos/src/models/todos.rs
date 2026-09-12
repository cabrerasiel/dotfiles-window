use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: Uuid,
    pub text: String,
    pub is_done: bool,
}

impl Todo {
    pub fn new(text: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            text,
            is_done: false,
        }
    }
}
