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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub id: Uuid,
    pub name: String,
    pub todos: Vec<Todo>,
}

impl Section {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            todos: Vec::new(),
        }
    }
}
