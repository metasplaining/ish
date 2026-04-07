*Extracted verbatim from [docs/project/proposals/refactoring-prototype.md](../../../proposals/refactoring-prototype.md) §Feature H1, M1, M2, M3, L1, with corrections for actual source types.*

---

## L1 — Exit-Code Constant

```rust
/// Sentinel exit code used when a process terminates without a numeric code
/// (e.g., killed by signal).
const MISSING_EXIT_CODE: i64 = -1;
```

Place after imports, near the top of `interpreter.rs`.

Replace both `unwrap_or(-1)` calls in `run_command_pipeline` with `unwrap_or(MISSING_EXIT_CODE)`.

---

## M3 — Brace-Scanning Helper

```rust
/// Scan forward from `start` in `chars` to find `}`. Returns the variable name
/// and the index of the character after `}`, or `None` if `}` is not found.
fn scan_to_close_brace(chars: &[char], start: usize) -> Option<(String, usize)> {
    let mut j = start;
    while j < chars.len() && chars[j] != '}' {
        j += 1;
    }
    if j < chars.len() {
        Some((chars[start..j].iter().collect(), j + 1))
    } else {
        None
    }
}
```

Place as a free function near `interpolate_shell_quoted` in `interpreter.rs`.

**Call sites:**
- `${var}` branch: `scan_to_close_brace(&chars, i + 2)` (start after `$` and `{`)
- `{var}` branch: `scan_to_close_brace(&chars, i + 1)` (start after `{`)

**Unwrap removal in `{var}` branch:**
```rust
// Before:
name.chars().next().unwrap().is_ascii_alphabetic() || name.starts_with('_')
// After:
name.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
```

---

## M2 — Builtin Helpers

```rust
/// Check that `args` has exactly `n` elements; return a structured error otherwise.
fn arity(name: &str, args: &[Value], n: usize) -> Result<(), RuntimeError> {
    if args.len() != n {
        return Err(RuntimeError::system_error(
            format!("{} expects {} argument(s), got {}", name, n, args.len()),
            ErrorCode::ArgumentCountMismatch,
        ));
    }
    Ok(())
}

/// Create a builtin function value with no typed parameters (the common case).
fn new_builtin(
    name: &'static str,
    f: impl Fn(&[Value]) -> Result<Value, RuntimeError> + 'static,
) -> Value {
    new_compiled_function(name, vec![], vec![], None, f, Some(false))
}
```

Place after imports in `builtins.rs`, before `register_all`.

---

## M1 — AST Builtin Helper

```rust
fn simple_ast_builtin(
    name: &'static str,
    arity: usize,
    fields: &'static [&'static str],
) -> Value {
    use crate::value::new_compiled_function;
    new_compiled_function(name, vec![], vec![], None, move |args| {
        if args.len() != arity {
            return Err(RuntimeError::system_error(
                format!("{} expects {} argument(s)", name, arity),
                ErrorCode::TypeMismatch,
            ));
        }
        let mut map = HashMap::new();
        map.insert("kind".to_string(), str_val(
            name.strip_prefix("ast_").unwrap_or(name)
        ));
        for (field, arg) in fields.iter().zip(args.iter()) {
            map.insert(field.to_string(), arg.clone());
        }
        Ok(Value::Object(Gc::new(GcCell::new(map))))
    }, Some(false))
}
```

Place before `register_ast_builtins` in `reflection.rs`.

---

## H2 — ParseError Correction

**IMPORTANT:** `ParseError` in `proto/ish-parser/src/error.rs` uses fields `start: usize`,
`end: usize`, `message: String` — not `span`. Use `ParseError::new(start, end, message)`.

**Integer parse fix** (in `build_match_pattern`, `Rule::integer_literal` arm):
```rust
// Before:
let n: i64 = inner.as_str().parse().unwrap();
// After:
let s = inner.as_str();
let n: i64 = s.parse().map_err(|_| ParseError::new(0, 0,
    format!("integer literal '{}' overflows i64", s)
))?;
```

**Float parse fix** (in `build_match_pattern`, `Rule::float_literal` arm):
```rust
// Before:
let n: f64 = inner.as_str().parse().unwrap();
// After:
let s = inner.as_str();
let n: f64 = s.parse().map_err(|_| ParseError::new(0, 0,
    format!("float literal '{}' is not a valid f64", s)
))?;
```

**`lines.last()` fix** (in `strip_triple_quote_literal`):
```rust
// Before:
let last_line = lines.last().unwrap();
// After:
let empty = "";
let last_line = lines.last().copied().unwrap_or(empty);
```

**`value.unwrap()` fix** (in `build_var_decl`, around line ~120):
```rust
// Before:
value: value.unwrap(),
// After:
value: value.ok_or_else(|| ParseError::new(0, 0,
    "variable declaration missing value".to_string()
))?,
```
The `value` field is inside `Statement::VariableDecl { value, .. }` — the expression must
be placed where `value.unwrap()` currently appears in the struct literal.

---

## H1 — Six Interpreter Helpers

Place all six as private free functions in `interpreter.rs`, **below** the closing `}` of
`impl IshVm` and **above** the `#[cfg(test)]` section.

```rust
fn eval_literal(lit: &Literal) -> Value {
    match lit {
        Literal::Bool(b) => Value::Bool(*b),
        Literal::Int(n) => Value::Int(*n),
        Literal::Float(f) => Value::Float(*f),
        Literal::String(s) => Value::String(Rc::new(s.clone())),
        Literal::Char(c) => Value::Char(*c),
        Literal::Null => Value::Null,
    }
}

fn eval_unary_op(op: &UnaryOperator, val: Value) -> Result<Value, RuntimeError> {
    match op {
        UnaryOperator::Not => Ok(Value::Bool(!val.is_truthy())),
        UnaryOperator::Negate => match val {
            Value::Int(n) => Ok(Value::Int(-n)),
            Value::Float(f) => Ok(Value::Float(-f)),
            _ => Err(RuntimeError::system_error(
                format!("cannot negate {}", val.type_name()),
                ErrorCode::TypeMismatch,
            )),
        },
        UnaryOperator::Try => {
            if val == Value::Null {
                return Err(RuntimeError::system_error(
                    "tried to unwrap null value with ?".to_string(),
                    ErrorCode::NullUnwrap,
                ));
            }
            Ok(val)
        }
    }
}

fn apply_property_read(obj: Value, property: &str) -> Result<Value, RuntimeError> {
    match obj {
        Value::Object(ref obj_ref) => {
            let map = obj_ref.borrow();
            Ok(map.get(property).cloned().unwrap_or(Value::Null))
        }
        _ => Err(RuntimeError::system_error(
            format!("cannot access property '{}' on {}", property, obj.type_name()),
            ErrorCode::TypeMismatch,
        )),
    }
}

fn apply_index_read(obj: Value, idx: Value) -> Result<Value, RuntimeError> {
    match (&obj, &idx) {
        (Value::List(list_ref), Value::Int(i)) => {
            let list = list_ref.borrow();
            let i = *i;
            if i < 0 || i >= list.len() as i64 {
                return Err(RuntimeError::system_error(
                    format!("index {} out of bounds (length {})", i, list.len()),
                    ErrorCode::IndexOutOfBounds,
                ));
            }
            Ok(list[i as usize].clone())
        }
        (Value::Object(obj_ref), Value::String(key)) => {
            let map = obj_ref.borrow();
            Ok(map.get(key.as_ref()).cloned().unwrap_or(Value::Null))
        }
        _ => Err(RuntimeError::system_error(
            format!("cannot index {} with {}", obj.type_name(), idx.type_name()),
            ErrorCode::TypeMismatch,
        )),
    }
}

fn apply_property_write(obj: &Value, property: &str, val: Value) -> Result<(), RuntimeError> {
    if let Value::Object(ref obj_ref) = obj {
        obj_ref.borrow_mut().insert(property.to_string(), val);
        Ok(())
    } else {
        Err(RuntimeError::system_error(
            format!("cannot set property '{}' on {}", property, obj.type_name()),
            ErrorCode::TypeMismatch,
        ))
    }
}

fn apply_index_write(obj: &Value, idx: &Value, val: Value) -> Result<(), RuntimeError> {
    if let Value::List(ref list_ref) = obj {
        if let Value::Int(i) = idx {
            let mut list = list_ref.borrow_mut();
            let len = list.len() as i64;
            if *i < 0 || *i >= len {
                return Err(RuntimeError::system_error(
                    format!("index {} out of bounds (length {})", i, len),
                    ErrorCode::IndexOutOfBounds,
                ));
            }
            list[*i as usize] = val;
            Ok(())
        } else {
            Err(RuntimeError::system_error(
                "list index must be an integer",
                ErrorCode::TypeMismatch,
            ))
        }
    } else if let Value::Object(ref obj_ref) = obj {
        if let Value::String(ref key) = idx {
            obj_ref.borrow_mut().insert(key.as_ref().clone(), val);
            Ok(())
        } else {
            Err(RuntimeError::system_error(
                "object index must be a string",
                ErrorCode::TypeMismatch,
            ))
        }
    } else {
        Err(RuntimeError::system_error(
            format!("cannot index into {}", obj.type_name()),
            ErrorCode::TypeMismatch,
        ))
    }
}
```
