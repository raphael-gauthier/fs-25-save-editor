use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rotation {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalizedMessage {
    pub code: String,
    pub params: HashMap<String, String>,
}

impl LocalizedMessage {
    pub fn new(code: &str) -> Self {
        Self {
            code: code.to_string(),
            params: HashMap::new(),
        }
    }

    pub fn with_param(mut self, key: &str, value: impl ToString) -> Self {
        self.params.insert(key.to_string(), value.to_string());
        self
    }
}
