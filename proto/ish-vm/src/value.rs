use gc::{Finalize, Gc, GcCell, Trace};
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use ish_ast::Statement;
use crate::environment::Environment;
use crate::error::RuntimeError;

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
    BuiltinFunction(#[unsafe_ignore_trace] BuiltinRef),
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

#[derive(Clone, Debug, Trace, Finalize)]
pub struct IshFunction {
    pub name: Option<String>,
    #[unsafe_ignore_trace]
    pub params: Vec<String>,
    #[unsafe_ignore_trace]
    pub param_types: Vec<Option<ish_ast::TypeAnnotation>>,
    #[unsafe_ignore_trace]
    pub return_type: Option<ish_ast::TypeAnnotation>,
    #[unsafe_ignore_trace]
    pub body: Statement, // must be a Block
    pub closure_env: Environment,
}

pub type FunctionRef = Gc<IshFunction>;

pub fn new_function(
    name: Option<String>,
    params: Vec<String>,
    param_types: Vec<Option<ish_ast::TypeAnnotation>>,
    return_type: Option<ish_ast::TypeAnnotation>,
    body: Statement,
    closure_env: Environment,
) -> Value {
    Value::Function(Gc::new(IshFunction {
        name,
        params,
        param_types,
        return_type,
        body,
        closure_env,
    }))
}

// ── Builtin Function ────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct BuiltinFn {
    pub name: String,
    pub func: Rc<dyn Fn(&[Value]) -> Result<Value, RuntimeError>>,
}

impl fmt::Debug for BuiltinFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<builtin:{}>", self.name)
    }
}

pub type BuiltinRef = Rc<BuiltinFn>;

pub fn new_builtin(
    name: impl Into<String>,
    func: impl Fn(&[Value]) -> Result<Value, RuntimeError> + 'static,
) -> Value {
    Value::BuiltinFunction(Rc::new(BuiltinFn {
        name: name.into(),
        func: Rc::new(func),
    }))
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
            Value::BuiltinFunction(_) => "function",
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
            Value::BuiltinFunction(b) => format!("<builtin:{}>", b.name),
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
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_display_string())
    }
}
