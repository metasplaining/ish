---
title: "Architecture: ish-vm"
category: architecture
audience: [all]
status: draft
last-verified: 2026-03-10
depends-on: [docs/architecture/overview.md, docs/architecture/ast.md]
---

# ish-vm

**Source:** `proto/ish-vm/src/`

Tree-walking interpreter executing AST programs.

---

## Value System (`value.rs`)

```rust
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(Rc<String>),
    Null,
    Object(ObjectRef),              // Gc<GcCell<HashMap<String, Value>>>
    List(ListRef),                  // Gc<GcCell<Vec<Value>>>
    Function(FunctionRef),          // Gc<IshFunction>
    BuiltinFunction(BuiltinRef),    // Rc<BuiltinFn>
}
```

- **GC-managed:** Objects, Lists, and Functions use the `gc` crate (v0.5)
- **Strings** use `Rc<String>` (cheap cloning, no GC overhead for immutable data)
- **`PartialEq`** supports cross-type Int/Float comparison
- **`is_truthy()`** — false for `false`, `0`, `0.0`, `Null`; true for everything else

---

## Environment (`environment.rs`)

Lexical scoping via a chain of GC-managed scopes:

```rust
pub struct Environment {
    inner: Gc<GcCell<Scope>>,
}

struct Scope {
    vars: HashMap<String, Value>,
    parent: Option<Environment>,
}
```

- `Environment::new()` — root scope
- `env.child()` — create child scope
- `env.define(name, value)` — bind in current scope
- `env.get(name)` — walk chain upward
- `env.set(name, value)` — update existing binding (walks chain)

Closures capture an `Environment` at definition time. When a closure is called, a child of the captured environment becomes the function's local scope.

---

## Interpreter (`interpreter.rs`)

```rust
pub struct IshVm {
    pub global_env: Environment,
}
```

| Method | Description |
|--------|-------------|
| `IshVm::new()` | Creates VM, registers all builtins + AST factory functions |
| `vm.run(&Program)` | Execute a program, return last expression's value |
| `vm.eval_expression(&Expression, &Environment)` | Evaluate a single expression |
| `vm.call_function(&Value, &[Value])` | Call a function value with arguments |

**Control flow** uses `ControlFlow::None`, `ControlFlow::Return(Value)`, `ControlFlow::ExprValue(Value)`.

**Short-circuit evaluation:** `And` and `Or` operators only evaluate the right operand when needed.

**Division by zero** returns a `RuntimeError` rather than panicking.

---

## Builtins (`builtins.rs`)

45 Rust-native functions registered at VM startup. All take `&[Value]` and return `Result<Value, RuntimeError>`.

| Group | Functions |
|-------|-----------|
| I/O | `print`, `println`, `read_file`, `write_file` |
| Strings | `str_concat`, `str_length`, `str_slice`, `str_contains`, `str_starts_with`, `str_replace`, `str_split`, `str_to_upper`, `str_to_lower`, `str_char_at`, `str_trim` |
| Lists | `list_push`, `list_pop`, `list_length`, `list_get`, `list_set`, `list_slice`, `list_join` |
| Objects | `obj_get`, `obj_set`, `obj_has`, `obj_keys`, `obj_values`, `obj_remove` |
| Types | `type_of`, `is_type` |
| Conversion | `to_string`, `to_int`, `to_float` |

---

## Reflection (`reflection.rs`)

Bidirectional conversion between Rust AST types and ish Values (Objects with `"kind"` discriminators):

**AST → Value:** `program_to_value()`, `stmt_to_value()`, `expr_to_value()`

**Value → AST:** `value_to_program()`, `value_to_stmt()`, `value_to_expr()`

**AST factory builtins** (22 functions callable from ish programs):

`ast_program`, `ast_literal`, `ast_identifier`, `ast_binary_op`, `ast_unary_op`, `ast_function_call`, `ast_block`, `ast_return`, `ast_var_decl`, `ast_if`, `ast_while`, `ast_function_decl`, `ast_expr_stmt`, `ast_lambda`, `ast_property_access`, `ast_index_access`, `ast_object_literal`, `ast_list_literal`, `ast_param`, `ast_assignment`, `ast_assign_target_var`, `ast_for_each`

### Value representation of AST nodes

Every node is an Object with a `"kind"` field:

```json
{ "kind": "literal", "literal_type": "int", "value": 42 }
{ "kind": "identifier", "name": "x" }
{ "kind": "binary_op", "op": "add", "left": {...}, "right": {...} }
{ "kind": "var_decl", "name": "x", "value": {...} }
{ "kind": "function_decl", "name": "factorial", "params": [...], "body": {...} }
```

---

## Error Handling (`error.rs`)

`RuntimeError` type used throughout the VM.

---

## Tests

- `interpreter.rs`: 8 tests
- `builtins.rs`: 6 tests
- `reflection.rs`: 4 tests

---

## Referenced by

- [docs/architecture/INDEX.md](INDEX.md)
- [docs/architecture/overview.md](overview.md)
- [docs/architecture/stdlib.md](stdlib.md)
- [docs/spec/types.md](../spec/types.md)
