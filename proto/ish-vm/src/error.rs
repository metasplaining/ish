use std::fmt;
use std::rc::Rc;

use crate::value::{Value, new_object};

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

    /// Create a RuntimeError wrapping a SystemError object with the given
    /// message and error code.  The thrown value is an ish object with
    /// `message` and `code` properties, matching the entry-based error model.
    pub fn system_error(msg: impl Into<String>, code: impl Into<String>) -> Self {
        let msg = msg.into();
        let code = code.into();
        let mut map = std::collections::HashMap::new();
        map.insert("message".to_string(), Value::String(Rc::new(msg.clone())));
        map.insert("code".to_string(), Value::String(Rc::new(code)));
        RuntimeError {
            message: msg,
            thrown_value: Some(new_object(map)),
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RuntimeError: {}", self.message)
    }
}

impl std::error::Error for RuntimeError {}
