*Extracted verbatim from `proto/ish-vm/src/reflection.rs` lines 1047–1087 on 2026-04-06.*

---

## Three Builtins That Must NOT Use `simple_ast_builtin`

The following three builtins in `register_ast_builtins` deviate from the simple pattern and
must remain as inline closures:

### 1. `ast_literal` — Extra `literal_type` field

```rust
// ast_literal(value) -> literal node
env.define(
    "ast_literal".into(),
    new_compiled_function("ast_literal", vec![], vec![], None, |args| {
        if args.len() != 1 {
            return Err(RuntimeError::system_error("ast_literal expects 1 argument", ErrorCode::TypeMismatch));
        }
        let mut map = HashMap::new();
        map.insert("kind".to_string(), str_val("literal"));
        map.insert("value".to_string(), args[0].clone());
        let lit_type = match &args[0] {
            Value::Bool(_) => "bool",
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Char(_) => "char",
            Value::Null => "null",
            _ => "unknown",
        };
        map.insert("literal_type".to_string(), str_val(lit_type));
        Ok(Value::Object(Gc::new(GcCell::new(map))))
    }, Some(false)),
);
```

**Why:** Has an extra `literal_type` field populated by inspecting `args[0]`'s type.
`simple_ast_builtin` cannot express this — keep as-is.

---

### 2. `ast_param` — No `kind` field at all

```rust
// ast_param(name) -> parameter object
env.define(
    "ast_param".into(),
    new_compiled_function("ast_param", vec![], vec![], None, |args| {
        if args.len() != 1 {
            return Err(RuntimeError::system_error("ast_param expects 1 argument", ErrorCode::TypeMismatch));
        }
        let mut map = HashMap::new();
        map.insert("name".to_string(), args[0].clone());
        Ok(Value::Object(Gc::new(GcCell::new(map))))
    }, Some(false)),
);
```

**Why:** Produces `{"name": ...}` with no `"kind"` field. `simple_ast_builtin` always
inserts a `"kind"` field derived from the name. Keep as-is.

---

### 3. `ast_assign_target_var` — `kind` does NOT follow the naming convention

```rust
// ast_assign_target_var(name) -> assign target object
env.define(
    "ast_assign_target_var".into(),
    new_compiled_function("ast_assign_target_var", vec![], vec![], None, |args| {
        if args.len() != 1 {
            return Err(RuntimeError::system_error("ast_assign_target_var expects 1 argument", ErrorCode::TypeMismatch));
        }
        let mut map = HashMap::new();
        map.insert("kind".to_string(), str_val("variable"));
        map.insert("name".to_string(), args[0].clone());
        Ok(Value::Object(Gc::new(GcCell::new(map))))
    }, Some(false)),
);
```

**Why:** The `kind` field is `"variable"` but stripping `"ast_"` from `"ast_assign_target_var"`
gives `"assign_target_var"`. The name does not follow the convention. Keep as-is.

---

## Simple Builtins Table (use `simple_ast_builtin` for all of these)

| Builtin name | Arity | Fields (in order) |
|---|---|---|
| `ast_program` | 1 | `["statements"]` |
| `ast_identifier` | 1 | `["name"]` |
| `ast_binary_op` | 3 | `["op", "left", "right"]` |
| `ast_unary_op` | 2 | `["op", "operand"]` |
| `ast_function_call` | 2 | `["callee", "args"]` |
| `ast_block` | 1 | `["statements"]` |
| `ast_return` | 1 | `["value"]` |
| `ast_var_decl` | 2 | `["name", "value"]` |
| `ast_if` | 3 | `["condition", "then_block", "else_block"]` |
| `ast_while` | 2 | `["condition", "body"]` |
| `ast_function_decl` | 3 | `["name", "params", "body"]` |
| `ast_expr_stmt` | 1 | `["expression"]` |
| `ast_lambda` | 2 | `["params", "body"]` |
| `ast_property_access` | 2 | `["object", "property"]` |
| `ast_index_access` | 2 | `["object", "index"]` |
| `ast_object_literal` | 1 | `["pairs"]` |
| `ast_list_literal` | 1 | `["elements"]` |
| `ast_assignment` | 2 | `["target", "value"]` |
| `ast_for_each` | 3 | `["variable", "iterable", "body"]` |
| `ast_throw` | 1 | `["value"]` |
| `ast_try_catch` | 3 | `["body", "catches", "finally"]` |
| `ast_defer` | 1 | `["body"]` |
