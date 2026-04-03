use gc::{Finalize, Gc, GcCell, Trace};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::error::RuntimeError;

/// Thread-local set of future IDs that have been spawned but not awaited.
thread_local! {
    static UNAWAITED_FUTURES: RefCell<Vec<u64>> = RefCell::new(Vec::new());
    static NEXT_FUTURE_ID: RefCell<u64> = RefCell::new(0);
}

fn register_future() -> u64 {
    NEXT_FUTURE_ID.with(|id| {
        let current = *id.borrow();
        *id.borrow_mut() = current + 1;
        UNAWAITED_FUTURES.with(|set| set.borrow_mut().push(current));
        current
    })
}

fn mark_future_awaited(id: u64) {
    UNAWAITED_FUTURES.with(|set| {
        set.borrow_mut().retain(|&i| i != id);
    });
}

/// Get the count of futures that were spawned but never awaited, and reset.
pub fn take_unawaited_future_count() -> usize {
    UNAWAITED_FUTURES.with(|set| {
        let count = set.borrow().len();
        set.borrow_mut().clear();
        count
    })
}

/// Core runtime value type for the ish interpreter.
#[derive(Clone, Debug, Trace, Finalize)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(#[unsafe_ignore_trace] Rc<String>),
    Char(char),
    Null,
    Object(ObjectRef),
    List(ListRef),
    Function(FunctionRef),
    Future(#[unsafe_ignore_trace] FutureRef),
}

// ── Object ──────────────────────────────────────────────────────────────────

pub type ObjectRef = Gc<GcCell<HashMap<String, Value>>>;

pub fn new_object(map: HashMap<String, Value>) -> Value {
    Value::Object(Gc::new(GcCell::new(map)))
}

pub fn empty_object() -> Value {
    new_object(HashMap::new())
}

// ── List ────────────────────────────────────────────────────────────────────

pub type ListRef = Gc<GcCell<Vec<Value>>>;

pub fn new_list(items: Vec<Value>) -> Value {
    Value::List(Gc::new(GcCell::new(items)))
}

// ── Function ────────────────────────────────────────────────────────────────

/// Synchronous shim function: receives validated args, returns a Value (which may be Future).
pub type Shim = Rc<dyn Fn(&[Value]) -> Result<Value, RuntimeError>>;

#[derive(Clone, Trace, Finalize)]
pub struct IshFunction {
    pub name: Option<String>,
    #[unsafe_ignore_trace]
    pub params: Vec<String>,
    #[unsafe_ignore_trace]
    pub param_types: Vec<Option<ish_core::TypeAnnotation>>,
    #[unsafe_ignore_trace]
    pub return_type: Option<ish_core::TypeAnnotation>,
    #[unsafe_ignore_trace]
    pub shim: Shim,
    #[unsafe_ignore_trace]
    pub is_async: bool,
    /// Yielding classification: Some(true) = yielding, Some(false) = unyielding, None = unclassified.
    #[unsafe_ignore_trace]
    pub has_yielding_entry: Option<bool>,
}

impl fmt::Debug for IshFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IshFunction")
            .field("name", &self.name)
            .field("params", &self.params)
            .field("is_async", &self.is_async)
            .field("has_yielding_entry", &self.has_yielding_entry)
            .finish()
    }
}

pub type FunctionRef = Gc<IshFunction>;

/// Create a compiled (shim-based) function value.
pub fn new_compiled_function(
    name: impl Into<String>,
    params: Vec<String>,
    param_types: Vec<Option<ish_core::TypeAnnotation>>,
    return_type: Option<ish_core::TypeAnnotation>,
    shim: impl Fn(&[Value]) -> Result<Value, RuntimeError> + 'static,
    has_yielding_entry: Option<bool>,
) -> Value {
    Value::Function(Gc::new(IshFunction {
        name: Some(name.into()),
        params,
        param_types,
        return_type,
        shim: Rc::new(shim),
        is_async: false,
        has_yielding_entry,
    }))
}

// ── Future ──────────────────────────────────────────────────────────────────

/// A reference to a spawned task's JoinHandle. When dropped without being
/// awaited, the underlying task is cancelled via `abort()`.
#[derive(Clone)]
pub struct FutureRef {
    inner: Rc<RefCell<Option<tokio::task::JoinHandle<Result<Value, RuntimeError>>>>>,
    id: u64,
}

impl FutureRef {
    pub fn new(handle: tokio::task::JoinHandle<Result<Value, RuntimeError>>) -> Self {
        let id = register_future();
        FutureRef {
            inner: Rc::new(RefCell::new(Some(handle))),
            id,
        }
    }

    /// Take the JoinHandle out (for awaiting). Returns None if already taken.
    pub fn take(&self) -> Option<tokio::task::JoinHandle<Result<Value, RuntimeError>>> {
        let handle = self.inner.borrow_mut().take();
        if handle.is_some() {
            mark_future_awaited(self.id);
        }
        handle
    }
}

impl fmt::Debug for FutureRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.inner.borrow().is_some() {
            "pending"
        } else {
            "consumed"
        };
        write!(f, "<future:{}>", status)
    }
}

impl Drop for FutureRef {
    fn drop(&mut self) {
        // Only abort if this is the last reference and handle hasn't been taken (awaited)
        if Rc::strong_count(&self.inner) == 1 {
            if let Some(handle) = self.inner.borrow_mut().take() {
                handle.abort();
            }
        }
    }
}

// ── Value methods ───────────────────────────────────────────────────────────

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Null => false,
            Value::Int(0) => false,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Char(_) => true,
            _ => true,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Bool(_) => "bool",
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Char(_) => "char",
            Value::Null => "null",
            Value::Object(_) => "object",
            Value::List(_) => "list",
            Value::Function(_) => "function",
            Value::Future(_) => "future",
        }
    }

    pub fn to_display_string(&self) -> String {
        match self {
            Value::Bool(b) => b.to_string(),
            Value::Int(n) => n.to_string(),
            Value::Float(f) => {
                if f.fract() == 0.0 {
                    format!("{:.1}", f)
                } else {
                    f.to_string()
                }
            }
            Value::String(s) => s.as_ref().clone(),
            Value::Char(c) => c.to_string(),
            Value::Null => "null".to_string(),
            Value::Object(obj) => {
                let map = obj.borrow();
                let pairs: Vec<String> = map
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_display_string()))
                    .collect();
                format!("{{ {} }}", pairs.join(", "))
            }
            Value::List(list) => {
                let items = list.borrow();
                let elems: Vec<String> = items.iter().map(|v| v.to_display_string()).collect();
                format!("[{}]", elems.join(", "))
            }
            Value::Function(f) => {
                if let Some(name) = &f.name {
                    format!("<fn:{}>", name)
                } else {
                    "<fn:anonymous>".to_string()
                }
            }
            Value::Future(_) => "<future>".to_string(),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Int(a), Value::Float(b)) => (*a as f64) == *b,
            (Value::Float(a), Value::Int(b)) => *a == (*b as f64),
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Char(a), Value::Char(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::Object(a), Value::Object(b)) => {
                let a = a.borrow();
                let b = b.borrow();
                *a == *b
            }
            (Value::List(a), Value::List(b)) => {
                let a = a.borrow();
                let b = b.borrow();
                *a == *b
            }
            // Futures use identity equality (same Rc pointer)
            (Value::Future(a), Value::Future(b)) => Rc::ptr_eq(&a.inner, &b.inner),
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_display_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── FutureRef identity equality (TODO 47) ───────────────────────────

    #[test]
    fn future_same_ref_is_equal() {
        let handle = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(async {
                let local = tokio::task::LocalSet::new();
                local.run_until(async {
                    let h = tokio::task::spawn_local(async { Ok(Value::Null) });
                    FutureRef::new(h)
                }).await
            });
        let a = Value::Future(handle.clone());
        let b = Value::Future(handle);
        assert_eq!(a, b);
    }

    #[test]
    fn future_different_refs_not_equal() {
        let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
        let (f1, f2) = rt.block_on(async {
            let local = tokio::task::LocalSet::new();
            local.run_until(async {
                let h1 = tokio::task::spawn_local(async { Ok(Value::Null) });
                let h2 = tokio::task::spawn_local(async { Ok(Value::Null) });
                (FutureRef::new(h1), FutureRef::new(h2))
            }).await
        });
        assert_ne!(Value::Future(f1), Value::Future(f2));
    }

    // ── new_compiled_function (TODO 48) ─────────────────────────────────

    #[test]
    fn compiled_function_creates_function_value() {
        let val = new_compiled_function(
            "test_fn",
            vec![],
            vec![],
            None,
            |_args| Ok(Value::Int(42)),
            Some(false),
        );
        assert_eq!(val.type_name(), "function");
    }

    #[test]
    fn compiled_function_has_yielding_entry() {
        let val = new_compiled_function(
            "unyielding",
            vec![],
            vec![],
            None,
            |_args| Ok(Value::Null),
            Some(false),
        );
        if let Value::Function(f) = &val {
            assert_eq!(f.has_yielding_entry, Some(false));
        } else {
            panic!("expected Function");
        }
    }

    #[test]
    fn compiled_function_display_shows_name() {
        let val = new_compiled_function(
            "my_builtin",
            vec![],
            vec![],
            None,
            |_args| Ok(Value::Null),
            Some(false),
        );
        assert_eq!(val.to_display_string(), "<fn:my_builtin>");
    }
}
