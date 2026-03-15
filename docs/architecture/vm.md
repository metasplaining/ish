---
title: "Architecture: ish-vm"
category: architecture
audience: [all]
status: draft
last-verified: 2026-03-11
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
    Char(char),
    Null,
    Object(ObjectRef),              // Gc<GcCell<HashMap<String, Value>>>
    List(ListRef),                  // Gc<GcCell<Vec<Value>>>
    Function(FunctionRef),          // Gc<IshFunction>
    BuiltinFunction(BuiltinRef),    // Rc<BuiltinFn>
}
```

- **GC-managed:** Objects, Lists, and Functions use the `gc` crate (v0.5)
- **Strings** use `Rc<String>` (cheap cloning, no GC overhead for immutable data)
- **`PartialEq`** supports cross-type Int/Float comparison and Char equality
- **`is_truthy()`** — false for `false`, `0`, `0.0`, `Null`; true for everything else (including `Char`)

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

**Control flow** uses `ControlFlow::None`, `ControlFlow::Return(Value)`, `ControlFlow::ExprValue(Value)`, `ControlFlow::Throw(Value)`.

**Short-circuit evaluation:** `And` and `Or` operators only evaluate the right operand when needed.

**Division by zero** returns a `RuntimeError` rather than panicking.

### Throw and Try/Catch

The `Throw` statement evaluates its expression and returns `ControlFlow::Throw(value)`. This unwinds through blocks, loops, and other statements until it reaches either:

- A `TryCatch` statement, which catches the throw, binds the value to the catch clause's parameter, and executes the catch body.
- A function boundary, where `call_function` converts `ControlFlow::Throw(v)` into `Err(RuntimeError::thrown(v))`. The `TryCatch` handler also catches these `RuntimeError`s with `thrown_value`, so try/catch works across function calls.

`Finally` blocks always execute. A throw from a finally block replaces any in-flight error.

### With Blocks

`WithBlock` initializes resources in declaration order, executes the body, then calls `close()` on each resource in reverse order. If initialization of a later resource fails, earlier ones are closed. Body errors take precedence over close errors.

### Defer

`Defer` statements within a `Block` are collected during execution and run in LIFO order when the block exits — whether normally, via return, or via throw.

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
| Conversion | `to_string`, `to_int`, `to_float`, `char` |
| Errors | `new_error`, `is_error`, `error_message` |

---

## Reflection (`reflection.rs`)

Bidirectional conversion between Rust AST types and ish Values (Objects with `"kind"` discriminators):

**AST → Value:** `program_to_value()`, `stmt_to_value()`, `expr_to_value()`

**Value → AST:** `value_to_program()`, `value_to_stmt()`, `value_to_expr()`

**AST factory builtins** (22 functions callable from ish programs):

`ast_program`, `ast_literal`, `ast_identifier`, `ast_binary_op`, `ast_unary_op`, `ast_function_call`, `ast_block`, `ast_return`, `ast_var_decl`, `ast_if`, `ast_while`, `ast_function_decl`, `ast_expr_stmt`, `ast_lambda`, `ast_property_access`, `ast_index_access`, `ast_object_literal`, `ast_list_literal`, `ast_param`, `ast_assignment`, `ast_assign_target_var`, `ast_for_each`, `ast_throw`, `ast_try_catch`, `ast_defer`

### Value representation of AST nodes

Every node is an Object with a `"kind"` field:

```json
{ "kind": "literal", "literal_type": "int", "value": 42 }
{ "kind": "literal", "literal_type": "char", "value": "A" }
{ "kind": "identifier", "name": "x" }
{ "kind": "binary_op", "op": "add", "left": {...}, "right": {...} }
{ "kind": "var_decl", "name": "x", "value": {...} }
{ "kind": "function_decl", "name": "factorial", "params": [...], "body": {...} }
```

---

## Error Handling (`error.rs`)

`RuntimeError` type used throughout the VM. Contains a `message` field and an optional `thrown_value: Option<Value>` that preserves the original value when a throw crosses a function boundary.

- `RuntimeError::new(message)` — create a runtime error with a message
- `RuntimeError::thrown(value)` — create a runtime error from a thrown value, preserving it for the caller's try/catch

---

## Tests

- `interpreter.rs`: 19 tests (execution) + 14 error handling tests + 8 char/string syntax tests
- `builtins.rs`: 6 tests
- `reflection.rs`: 4 tests

---

## Referenced by

- [docs/architecture/INDEX.md](INDEX.md)
- [docs/architecture/overview.md](overview.md)
- [docs/architecture/stdlib.md](stdlib.md)
- [docs/spec/types.md](../spec/types.md)
