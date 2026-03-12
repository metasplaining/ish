use std::fmt;

use crate::value::Value;

#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub message: String,
    /// When the error was created from an ish `throw` statement,
    /// this carries the original thrown Value.
    pub thrown_value: Option<Value>,
}

impl RuntimeError {
    pub fn new(msg: impl Into<String>) -> Self {
        RuntimeError { message: msg.into(), thrown_value: None }
    }

    /// Create a RuntimeError that wraps a thrown ish Value.
    pub fn thrown(value: Value) -> Self {
        let message = format!("Thrown: {}", value.to_display_string());
        RuntimeError { message, thrown_value: Some(value) }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RuntimeError: {}", self.message)
    }
}

impl std::error::Error for RuntimeError {}
