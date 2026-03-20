use gc::{Finalize, Gc, GcCell, Trace};
use std::collections::HashMap;

use crate::value::Value;
use crate::error::RuntimeError;

/// A single scope containing variable bindings.
#[derive(Debug, Clone, Trace, Finalize)]
struct Scope {
    vars: HashMap<String, Value>,
    parent: Option<Environment>,
}

/// GC-managed environment with lexical scoping via parent chain.
#[derive(Debug, Clone, Trace, Finalize)]
pub struct Environment {
    inner: Gc<GcCell<Scope>>,
}

impl Environment {
    /// Create a new root environment with no parent.
    pub fn new() -> Self {
        Environment {
            inner: Gc::new(GcCell::new(Scope {
                vars: HashMap::new(),
                parent: None,
            })),
        }
    }

    /// Create a child environment that inherits from this one.
    pub fn child(&self) -> Self {
        Environment {
            inner: Gc::new(GcCell::new(Scope {
                vars: HashMap::new(),
                parent: Some(self.clone()),
            })),
        }
    }

    /// Define a new variable in the current scope.
    pub fn define(&self, name: String, value: Value) {
        self.inner.borrow_mut().vars.insert(name, value);
    }

    /// Look up a variable by walking the scope chain.
    pub fn get(&self, name: &str) -> Result<Value, RuntimeError> {
        let scope = self.inner.borrow();
        if let Some(val) = scope.vars.get(name) {
            return Ok(val.clone());
        }
        if let Some(ref parent) = scope.parent {
            return parent.get(name);
        }
        Err(RuntimeError::system_error(format!("undefined variable: {}", name), "E005"))
    }

    /// Set (re-assign) an existing variable by walking the scope chain.
    pub fn set(&self, name: &str, value: Value) -> Result<(), RuntimeError> {
        let mut scope = self.inner.borrow_mut();
        if scope.vars.contains_key(name) {
            scope.vars.insert(name.to_string(), value);
            return Ok(());
        }
        if let Some(ref parent) = scope.parent {
            return parent.set(name, value);
        }
        Err(RuntimeError::system_error(format!("undefined variable: {}", name), "E005"))
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}
