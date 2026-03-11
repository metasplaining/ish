use std::fmt;

#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub message: String,
}

impl RuntimeError {
    pub fn new(msg: impl Into<String>) -> Self {
        RuntimeError { message: msg.into() }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RuntimeError: {}", self.message)
    }
}

impl std::error::Error for RuntimeError {}
