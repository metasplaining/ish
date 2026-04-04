use std::fmt;
use std::rc::Rc;

use crate::value::{Value, new_object};

/// Type-safe error code enum identifying the category of a RuntimeError.
/// Each variant corresponds to an error code (E001–E013) in the error catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    UnhandledThrow,        // E001
    DivisionByZero,        // E002
    ArgumentCountMismatch, // E003
    TypeMismatch,          // E004
    UndefinedVariable,     // E005
    NotCallable,           // E006
    IndexOutOfBounds,      // E007
    IoError,               // E008
    NullUnwrap,            // E009
    ShellError,            // E010
    AsyncError,            // E011
    AwaitUnyielding,       // E012
    SpawnUnyielding,       // E013
    AwaitNonFuture,        // E014
    UnyieldingViolation,   // E015
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::UnhandledThrow => "E001",
            ErrorCode::DivisionByZero => "E002",
            ErrorCode::ArgumentCountMismatch => "E003",
            ErrorCode::TypeMismatch => "E004",
            ErrorCode::UndefinedVariable => "E005",
            ErrorCode::NotCallable => "E006",
            ErrorCode::IndexOutOfBounds => "E007",
            ErrorCode::IoError => "E008",
            ErrorCode::NullUnwrap => "E009",
            ErrorCode::ShellError => "E010",
            ErrorCode::AsyncError => "E011",
            ErrorCode::AwaitUnyielding => "E012",
            ErrorCode::SpawnUnyielding => "E013",
            ErrorCode::AwaitNonFuture => "E014",
            ErrorCode::UnyieldingViolation => "E015",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

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
    pub fn system_error(msg: impl Into<String>, code: ErrorCode) -> Self {
        let msg = msg.into();
        let code_str = code.as_str().to_string();
        let mut map = std::collections::HashMap::new();
        map.insert("message".to_string(), Value::String(Rc::new(msg.clone())));
        map.insert("code".to_string(), Value::String(Rc::new(code_str)));
        RuntimeError {
            message: msg,
            thrown_value: Some(new_object(map)),
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // If the thrown value is a coded error object (has a "code" field),
        // include the code in the display so that shell output contains it.
        if let Some(Value::Object(ref obj_ref)) = self.thrown_value {
            let map = obj_ref.borrow();
            if let Some(Value::String(ref code)) = map.get("code") {
                return write!(f, "RuntimeError: {} ({})", self.message, code);
            }
        }
        write!(f, "RuntimeError: {}", self.message)
    }
}

impl std::error::Error for RuntimeError {}
