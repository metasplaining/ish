---
title: "Architecture: ish-vm"
category: architecture
audience: [all]
status: draft
last-verified: 2026-04-02
depends-on: [docs/architecture/overview.md, docs/architecture/ast.md, docs/architecture/runtime.md, docs/spec/concurrency.md]
---

# ish-vm

**Source:** `proto/ish-vm/src/`

Tree-walking interpreter executing AST programs. Value types (`Value`, `Shim`, `RuntimeError`, `ErrorCode`, `IshFunction`) are defined in `ish-runtime` and re-exported by this crate.

---

## Value System

See [runtime.md](runtime.md) for the full `Value` enum, `Shim` type alias, `IshFunction` struct, and `RuntimeError`/`ErrorCode` types. This crate re-exports them for backward compatibility.

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

### Rc<RefCell<IshVm>> Pattern

The VM is accessed via `Rc<RefCell<IshVm>>` throughout the interpreter, builtins, and shell entry points. Methods are associated functions that take `vm: &Rc<RefCell<IshVm>>` rather than `&mut self`. The key discipline: borrow mutably only for the specific mutation, then release before any call that might invoke a shim (which may re-enter the VM).

| Method | Description |
|--------|-------------|
| `IshVm::new()` | Creates VM, registers all builtins + AST factory functions |
| `IshVm::run(vm, &Program)` | Execute a program, return last expression's value |
| `IshVm::eval_expression_yielding(vm, &Expression, &Environment)` | Evaluate a single expression (async) |
| `IshVm::eval_expression_unyielding(vm, &Expression, &Environment)` | Evaluate a single expression (sync) |
| `IshVm::call_function(vm, &Value, &[Value])` | Call a function value with arguments (sync) |

### Shim-Only Function Dispatch

All functions — builtins and interpreted — are called uniformly via `(func.shim)(args)`. There is no dispatch on implementation variant. Shims are self-contained: they capture everything needed to execute the function body.

When the interpreter declares an interpreted function (`Statement::FunctionDecl`, `Expression::Lambda`), the code analyzer classifies it as yielding or unyielding, then the VM creates the appropriate shim:

- **Unyielding shims** capture the VM reference, body statement, closure environment, and parameter names. When called, the shim creates a child scope, binds parameters, and calls `exec_statement_unyielding` synchronously to execute the body.
- **Yielding shims** capture the same data. When called, the shim uses `tokio::task::spawn_local` to spawn an async task that executes the body via `exec_statement_yielding`, wraps the `JoinHandle` in a `FutureRef`, and returns `Value::Future`.

Arity checking, parameter type auditing, and return type auditing still occur around shim invocation in `call_function_inner`, which is synchronous.

**Control flow** uses `ControlFlow::None`, `ControlFlow::Return(Value)`, `ControlFlow::ExprValue(Value)`, `ControlFlow::Throw(Value)`.

### Code Analyzer (`analyzer.rs`)

The code analyzer classifies functions as yielding or unyielding at declaration time. It is a stub implementation — future versions will add type inference, reachability analysis, and other passes.

The analyzer walks the function body AST looking for yielding nodes:
- `Expression::Await`, `Expression::Spawn`, `Statement::Yield`, `Expression::CommandSubstitution`
- `Expression::FunctionCall` where the callee resolves to a yielding function in the environment

Functions declared `async` are immediately classified as yielding without body analysis. The analyzer does not recurse into nested `FunctionDecl` or `Lambda` bodies — those are classified independently when they are declared.

**Known limitations:**
- No forward references: all functions must be declared before they are called.
- No call cycles: mutually recursive functions are not supported.
- Only direct `Identifier` calls are analyzed; indirect calls (through variables, higher-order functions) are not checked.

### Execution Variants

The interpreter has two parallel execution paths:

| Variant | Functions | Description |
|---------|-----------|-------------|
| Yielding | `exec_statement_yielding`, `eval_expression_yielding` | Async (`Pin<Box<dyn Future>>`). Supports `await`, `spawn`, `yield`, command substitution. Performs implied await on bare function calls that return `Value::Future`. |
| Unyielding | `exec_statement_unyielding`, `eval_expression_unyielding` | Synchronous. Errors on `await`, `spawn`, `yield`, and command substitution nodes. No `YieldContext` parameter. |

Both variants share extracted helper functions for pure computations (binary/unary ops, variable definition, etc.). The yielding variant is used by `run()` and yielding shims; the unyielding variant is used by unyielding shims.

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
- A function boundary, where `call_function_inner` converts `ControlFlow::Throw(v)` into `Err(RuntimeError::thrown(v))`. The `TryCatch` handler also catches these `RuntimeError`s with `thrown_value`, so try/catch works across function calls.

The throw audit only adds `@Error` entries. Other error classifications (`CodedError`, `TypeError`, `FileError`, etc.) are ordinary ish types recognized structurally by the type system, not by the throw audit.

`Finally` blocks always execute. A throw from a finally block replaces any in-flight error.

### With Blocks

`WithBlock` initializes resources in declaration order, executes the body, then calls `close()` on each resource in reverse order. If initialization of a later resource fails, earlier ones are closed. Body errors take precedence over close errors.

### Defer

`Defer` statements within a function are collected during execution and run in LIFO order when the function exits — whether normally, via return, or via throw.

---

## Builtins (`builtins.rs`)

49 Rust-native functions registered at VM startup as `IshFunction` values with compiled shims. All builtins are `Value::Function` — there is no separate `BuiltinFunction` type. To an outside observer, builtins are indistinguishable from user-defined functions.

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

**Unified dispatch:** `call_function_inner` handles all functions via a single `Value::Function` match arm. It is synchronous — shims handle their own async execution internally. Arity checking and parameter type auditing apply uniformly to builtins and user-defined functions. All functions are invoked via `(func.shim)(args)` — there is no dispatch on implementation variant.

**Ledger builtins** need VM access (they query `self.ledger`), so they are intercepted by name in `call_function_inner` *before* the shim invocation. Stub shims are registered so the names are callable and metadata is available; reaching the stub body is an error.

**`apply`** is a normal compiled function — it calls `(f.shim)(&args)` directly. If the target function is yielding, `apply` returns `Value::Future`; if unyielding, it returns the result directly. There is no special-case intercept for `apply`.

**Implied await:** When `call_function_inner` returns `Value::Future` from a bare function call (not under `await` or `spawn`), the yielding `eval_expression` path awaits the future automatically. In the unyielding path, `Value::Future` is returned as-is (this would only occur if the caller explicitly invokes a yielding shim through `apply`). The `await_required` feature check applies at higher assurance levels.

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

## Error Handling

`RuntimeError` and `ErrorCode` are defined in `ish-runtime` (see [runtime.md](runtime.md)). The VM uses `ErrorCode` variants for all system error construction. The error hierarchy uses a structural model: only `@Error` is a predefined entry type. `CodedError`, `SystemError`, `TypeError`, and other categories are ordinary ish types defined structurally. See [docs/spec/errors.md](../spec/errors.md).

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
