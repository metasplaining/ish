use std::fmt;

/// A parse error with location and message.
#[derive(Debug, Clone)]
pub struct ParseError {
    pub start: usize,
    pub end: usize,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}..{}] {}", self.start, self.end, self.message)
    }
}

impl std::error::Error for ParseError {}

impl ParseError {
    pub fn new(start: usize, end: usize, message: impl Into<String>) -> Self {
        Self { start, end, message: message.into() }
    }
}
