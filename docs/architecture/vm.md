---
title: "Architecture: ish-vm"
category: architecture
audience: [all]
status: draft
last-verified: 2026-03-31
depends-on: [docs/architecture/overview.md, docs/architecture/ast.md, docs/spec/concurrency.md]
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
    Future(FutureRef),              // Rc<RefCell<Option<JoinHandle>>>
}
```

- **GC-managed:** Objects, Lists, and Functions use the `gc` crate (v0.5)
- **Strings** use `Rc<String>` (cheap cloning, no GC overhead for immutable data)
- **`PartialEq`** supports cross-type Int/Float comparison, Char equality, and Future identity equality (`Rc::ptr_eq`)
- **`is_truthy()`** — false for `false`, `0`, `0.0`, `Null`; true for everything else (including `Char`)
- **No `BuiltinFunction` variant** — all builtins are `Function` values with compiled implementations

### Function Implementation

Functions use a `FunctionImplementation` enum to distinguish interpreted from compiled execution:

```rust
pub enum FunctionImplementation {
    Interpreted(Statement),       // Tree-walking via exec_statement
    Compiled(Shim),               // Synchronous shim function
}

pub type Shim = Rc<dyn Fn(&[Value]) -> Result<Value, RuntimeError>>;
```

`IshFunction` carries the implementation along with metadata:

```rust
pub struct IshFunction {
    pub name: Option<String>,
    pub params: Vec<String>,
    pub param_types: Vec<Option<TypeAnnotation>>,
    pub return_type: Option<TypeAnnotation>,
    pub implementation: FunctionImplementation,
    pub closure_env: Environment,
    pub is_async: bool,
    pub has_yielding_entry: Option<bool>,  // None=ambiguous, Some(true)=yielding, Some(false)=unyielding
}
```

**Shim types** (behavioral, not structural — all use the same `Shim` type):
- **Unyielding shims** (`len`, `type_of`, etc.) — call logic directly, return a plain `Value`.
- **Yielding shims** — spawn work via `spawn_local`, return `Value::Future`.
- **Parallel shims** (`print`, `read_file`, etc.) — marshal args to `Send`-safe form, `spawn_blocking` + `spawn_local` bridge, return `Value::Future`.

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
    pub defer_stack: Vec<Vec<(Statement, Environment)>>,
    pub ledger: LedgerState,
}
```

| Method | Description |
|--------|-------------|
| `IshVm::new()` | Creates VM, registers all builtins + AST factory functions |
| `vm.run(&Program)` | Execute a program, return last expression's value |
| `vm.eval_expression(&Expression, &Environment)` | Evaluate a single expression |
| `vm.call_function(&Value, &[Value])` | Call a function value with arguments |

**Control flow** uses `ControlFlow::None`, `ControlFlow::Return(Value)`, `ControlFlow::ExprValue(Value)`, `ControlFlow::Throw(Value)`.

### Async Execution

The interpreter's core `eval` function is `async`. All execution runs inside a Tokio `LocalSet` via `spawn_local`, keeping GC-managed values `!Send`-safe. Key implications:

- `spawn` creates a new task on the `LocalSet` via `tokio::task::spawn_local()`, returning a `Future<T>` value that wraps the `JoinHandle`.
- `await` suspends the current task, yielding control to the Tokio scheduler until the awaited future resolves.
- At low assurance, calls to async standard library functions are implicitly awaited.

### Yield Budget

At yield-eligible points (loop back-edges, function call sites, explicit `yield` statements), the interpreter checks a time-based yield budget (~1ms default). If the budget is exhausted, it inserts `tokio::task::yield_now().await` to give other tasks a chance to run. At higher assurance levels, `yield every N` (statement-count-based) and `@[yield_budget(Xus)]` (custom time threshold) provide fine-grained control.

### Future Value

The `Value` enum includes a `Future` variant wrapping a `JoinHandle` from `spawn_local`. When a `Future` is dropped without being awaited, `JoinHandle::abort()` cancels the underlying task. `defer` and `with` cleanup blocks still execute in cancelled tasks. Awaiting a cancelled future returns a cancellation error (E011).

### Output Routing

In interactive mode, `println` and expression result display route through Reedline's `ExternalPrinter` (writing to a channel that the shell thread reads). In non-interactive mode, output goes directly to OS stdout/stderr. Background task output (errors, println from spawned tasks) also goes through the same routing mechanism. See [shell.md](shell.md) for the two-thread architecture.

**Short-circuit evaluation:** `And` and `Or` operators only evaluate the right operand when needed.

**Division by zero** returns a `RuntimeError` rather than panicking.

### Throw and Try/Catch

The `Throw` statement evaluates its expression, performs a throw audit (auto-adds `@Error` entry if the value has `message: String` — see [docs/spec/errors.md](../spec/errors.md)), and returns `ControlFlow::Throw(value)`. This unwinds through blocks, loops, and other statements until it reaches either:

- A `TryCatch` statement, which catches the throw, binds the value to the catch clause's parameter, and executes the catch body.
- A function boundary, where `call_function` converts `ControlFlow::Throw(v)` into `Err(RuntimeError::thrown(v))`. The `TryCatch` handler also catches these `RuntimeError`s with `thrown_value`, so try/catch works across function calls.

The throw audit only adds `@Error` entries. Other error classifications (`CodedError`, `TypeError`, `FileError`, etc.) are ordinary ish types recognized structurally by the type system, not by the throw audit.

`Finally` blocks always execute. A throw from a finally block replaces any in-flight error.

### With Blocks

`WithBlock` initializes resources in declaration order, executes the body, then calls `close()` on each resource in reverse order. If initialization of a later resource fails, earlier ones are closed. Body errors take precedence over close errors.

### Defer

`Defer` statements within a function are collected during execution and run in LIFO order when the function exits — whether normally, via return, or via throw.

---

## Builtins (`builtins.rs`)

49 Rust-native functions registered at VM startup as `IshFunction` values with `Compiled(Shim)` implementations. All builtins are `Value::Function` — there is no separate `BuiltinFunction` type. To an outside observer, builtins are indistinguishable from user-defined functions.

| Group | Functions | Yielding |
|-------|-----------|----------|
| I/O | `print`, `println`, `read_file`, `write_file` | `Some(true)` — parallel shims, return `Value::Future` |
| Strings | `str_concat`, `str_length`, `str_slice`, `str_contains`, `str_starts_with`, `str_replace`, `str_split`, `str_to_upper`, `str_to_lower`, `str_char_at`, `str_trim` | `Some(false)` — unyielding |
| Lists | `list_push`, `list_pop`, `list_length`, `list_get`, `list_set`, `list_slice`, `list_join` | `Some(false)` |
| Objects | `obj_get`, `obj_set`, `obj_has`, `obj_keys`, `obj_values`, `obj_remove` | `Some(false)` |
| Types | `type_of`, `is_type` | `Some(false)` |
| Conversion | `to_string`, `to_int`, `to_float`, `char` | `Some(false)` |
| Errors | `is_error`, `error_message`, `error_code` | `Some(false)` |
| Ledger | `active_standard`, `feature_state`, `has_standard`, `has_entry_type` | `Some(false)` |

**Unified dispatch:** `call_function_inner` handles all functions via a single `Value::Function` match arm. Arity checking and parameter type auditing apply uniformly to builtins and user-defined functions. The match then dispatches on `FunctionImplementation::Interpreted` vs `Compiled`.

**Ledger builtins** need `&mut IshVm` access (they query `self.ledger`), so they are intercepted by name in `call_function_inner` *before* the implementation dispatch. Stub shims are registered so the names are callable and metadata is available; reaching the stub body is an error.

**Implied await:** When a `Compiled` shim returns `Value::Future` from a bare function call (not under `await` or `spawn`), the `FunctionCall` handler checks the `await_required` feature. If not active, the future is immediately awaited (implied await). If active, the future is returned as-is. This makes parallel builtins backward-compatible at low assurance.

**I/O completion:** Parallel shim futures do not resolve until the I/O operation is actually complete. In interactive mode, the shim sends output plus a `oneshot` acknowledgment channel; the future resolves only after the shell confirms the write.

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

## Assurance Ledger Runtime (`ledger/`)

The ledger runtime is a module within `ish-vm` that implements the assurance ledger engine. It is organized into five submodules:

| Module | Purpose |
|--------|---------|
| `standard.rs` | `FeatureState`, `Standard`, `StandardRegistry` — feature state model with two dimensions (`AnnotationDimension`, `AuditDimension`), standard definitions and inheritance, built-in standards |
| `entry_type.rs` | `EntryType`, `EntryTypeRegistry` — entry type definitions with inheritance and property requirements, built-in entry types |
| `entry.rs` | `Entry`, `EntrySet` — entry instances attached to values |
| `audit.rs` | Stateless `audit_statement()` function — checks a statement against active features and entries, returns `AuditResult` (Pass, AutoFix, Discrepancy) |
| `vm_integration.rs` | `LedgerState` — wires the engine into the VM: standard scope stack, entry store, registries |

### Architecture

The ledger uses a clean separation between engine and VM integration:

- **Engine** (`standard.rs`, `entry_type.rs`, `entry.rs`, `audit.rs`): Pure data structures and stateless logic. No dependency on the interpreter.
- **VM integration** (`vm_integration.rs`): Owns the mutable state — scope stack, entry store, registries. Exposed as `IshVm.ledger`.

The VM notifies the ledger of program events (variable declarations, assignments, branch points, function calls, throws). The ledger performs entry maintenance unconditionally and auditing per the active standard. The VM does not gate ledger operations based on feature states.

### Standard Scope Stack

When a `@standard[name]` annotation is encountered, the interpreter pushes the named standard onto the ledger's scope stack. On scope exit, it pops. The topmost standard determines active feature states. Features are resolved with inheritance — if a standard extends another, unspecified features fall through to the parent.

### Statement Handlers

- `Statement::StandardDef` — registers a standard in `self.ledger.standard_registry`
- `Statement::EntryTypeDef` — registers an entry type in `self.ledger.entry_type_registry`
- `Statement::Annotated` — pushes/pops standards for `@standard[name]` annotations around the inner statement

---

## Error Handling (`error.rs`)

`RuntimeError` type used throughout the VM. Contains a `message` field and an optional `thrown_value: Option<Value>` that preserves the original value when a throw crosses a function boundary.

- `RuntimeError::new(message)` — create a runtime error with a message
- `RuntimeError::thrown(value)` — create a runtime error from a thrown value, preserving it for the caller's try/catch

The error hierarchy uses a structural model: only `@Error` is a predefined entry type. `CodedError`, `SystemError`, `TypeError`, and other categories are ordinary ish types defined structurally. See [docs/spec/errors.md](../spec/errors.md).

---

## Tests

- `interpreter.rs`: 19 tests (execution) + 14 error handling tests + 8 char/string syntax tests
- `builtins.rs`: 6 tests
- `reflection.rs`: 4 tests
- `ledger/standard.rs`: 6 tests
- `ledger/entry_type.rs`: 7 tests
- `ledger/audit.rs`: 5 tests
- `ledger/vm_integration.rs`: 6 tests

---

## Referenced by

- [AGENTS.md](../../AGENTS.md)
- [docs/architecture/INDEX.md](INDEX.md)
- [docs/architecture/overview.md](overview.md)
- [docs/architecture/stdlib.md](stdlib.md)
- [docs/spec/types.md](../spec/types.md)
- [docs/spec/errors.md](../spec/errors.md)
- [docs/spec/assurance-ledger.md](../spec/assurance-ledger.md)
- [docs/spec/concurrency.md](../spec/concurrency.md)
