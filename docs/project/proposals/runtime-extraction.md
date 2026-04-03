---
title: "Proposal: Runtime Extraction"
category: proposal
audience: [all]
status: accepted
last-verified: 2026-04-02
depends-on: [docs/project/rfp/runtime-extraction.md, docs/architecture/runtime.md, docs/architecture/vm.md, docs/architecture/codegen.md, docs/spec/types.md]
---

# Proposal: Runtime Extraction

*Generated from [runtime-extraction.md](../rfp/runtime-extraction.md) on 2026-04-02.*

---

## Decision Register

All decisions made during design, consolidated here as the authoritative reference.

| # | Decision | Outcome |
|---|----------|---------|
| 1 | Strategy for moving `Value` from `ish-vm` to `ish-runtime` | Break the dependency on `Environment` and `Statement` first by making all functions shim-only. Then move the full transitive closure of `Value`. |
| 2 | Move `Shim` type alias from `ish-vm` to `ish-runtime` | Yes. Keep as a type alias. |
| 3 | Move `RuntimeError` from `ish-vm` to `ish-runtime` | Yes. |
| 4 | Move `Environment` from `ish-vm` to `ish-runtime` | No. `Environment` stays in `ish-vm`. The shim-only architecture removes the dependency from `IshFunction` → `Environment`. |
| 5 | Move `IshFunction` and related types from `ish-vm` to `ish-runtime` | Yes. `IshFunction` moves, but its `implementation` field is always a `Shim`. No `Interpreted` variant. No `closure_env` field. |
| 6 | Retire or keep `IshValue` after `Value` moves | Remove `IshValue`. YAGNI. |
| 7 | Move error code constants to `ish-runtime` | Yes. Add an `ErrorCode` enum now. |
| 8 | Add `gc` and `tokio` as dependencies of `ish-runtime` | Yes for `gc` and `tokio`. Not `crossbeam` (only needed by builtins). `ish-core` for `TypeAnnotation` (not the full `ish-ast`). |
| 9 | Shim-only `IshFunction` architecture | All functions are shim-based. The VM wraps closures and interpreted functions in shims at declaration time. `FunctionImplementation` enum is eliminated. |
| 10 | Testing strategy for shim nesting | Add a builtin that takes a function argument, producing interpreted→compiled→interpreted nesting. Test multiple layers deep. Skip cross-boundary error propagation testing for now. |
| 11 | VM self-reference strategy | Use `Rc<RefCell<IshVm>>` globally. Borrow mutably only when mutation is needed and release immediately. This replaces the current `&mut self` pattern throughout the interpreter. |
| 12 | `TypeAnnotation` dependency for `IshFunction` | Extract `TypeAnnotation` from `ish-ast` into a new `ish-core` crate. Both `ish-ast` and `ish-runtime` depend on `ish-core`. |

---

## Questions and Answers

### Q: Are there any other types beyond `Value`, `Shim`, and error codes that would be useful to compiled functions in ish packages?

Yes. The analysis identifies these types:

**Must move (transitive closure of `Value` after shim-only refactor):**

| Type | Current location | Why it must move |
|------|-----------------|------------------|
| `Value` | `ish-vm::value` | `Shim` signature: `Fn(&[Value]) -> Result<Value, RuntimeError>` |
| `RuntimeError` | `ish-vm::error` | `Shim` return type |
| `Shim` | `ish-vm::value` | Compiled packages define and call shims |
| `ObjectRef` | `ish-vm::value` | `Value::Object(ObjectRef)` — compiled code creates/reads objects |
| `ListRef` | `ish-vm::value` | `Value::List(ListRef)` — compiled code creates/reads lists |
| `IshFunction` / `FunctionRef` | `ish-vm::value` | `Value::Function(FunctionRef)` — shim-only, no `Environment` or `Statement` dependency |
| `FutureRef` | `ish-vm::value` | `Value::Future(FutureRef)` — yielding compiled functions return futures |

**Also moves:**

| Type | Current location | Rationale |
|------|-----------------|-----------|
| `ErrorCode` enum | new | Type-safe error code registry for compiled packages |
| `new_compiled_function()` | `ish-vm::value` | Factory for creating shim-based functions. The only function constructor after the refactor. |
| `new_object()`, `new_list()`, `empty_object()` | `ish-vm::value` | Convenience constructors for compiled code. |

**New crate (`ish-core`):**

| Type | Current location | Rationale |
|------|-----------------|-----------|
| `TypeAnnotation` | `ish-ast` | Referenced by `IshFunction` for param/return types. Extracted to `ish-core` so `ish-runtime` doesn't depend on `ish-ast`. |

**Does not move:**

| Type | Why not |
|------|---------|
| `Environment` | Interpreter-internal. The VM captures it inside the shim closure instead. |
| `FunctionImplementation` enum | Eliminated entirely. All functions use shims. |
| AST types (`Statement`, other AST nodes) | No longer referenced by `IshFunction`. Stay in `ish-ast`. |
| `IshVm` (interpreter) | Consumers of the runtime are compiled packages, not interpreters. |
| `LedgerState` | Compile-time and interpreter concern. |
| `ControlFlow` enum | Interpreter-internal dispatch mechanism. |

### Q: How does the VM pass a closure or interpreted function to a compiled function? How does the compiled function call it?

All functions are shims. When the VM encounters a function declaration (closure or top-level), it wraps the function body in a shim that captures:

1. The `Environment` (closure scope)
2. The `Statement` (function body)
3. An `Rc<RefCell<IshVm>>` reference to the VM

The resulting shim has the standard `Fn(&[Value]) -> Result<Value, RuntimeError>` signature. Compiled functions receive and call this shim through the normal `Value::Function` dispatch — they don't need to know whether the function was originally interpreted or compiled.

`IshFunction` no longer stores an `Environment` or `Statement`. It stores a `Shim`. The VM creates the shim at declaration time, capturing everything needed for execution inside the closure.

---

## Feature 1: Shim-Only Function Architecture

This feature is a prerequisite for the extraction. It refactors `IshFunction` to remove the `Interpreted` variant, breaking the dependency chain that would otherwise force `Environment`, `Statement`, and `ish-ast` into `ish-runtime`.

### Issues to Watch Out For

1. **VM self-reference in shim closures.** The shim for an interpreted function needs to call back into the VM. The decision is to use `Rc<RefCell<IshVm>>` globally, replacing the current `&mut self` pattern. Every mutable VM access borrows mutably, performs the mutation, and releases immediately. This makes it straightforward to verify no double-borrows occur: the VM releases its borrow before calling any shim, and the shim re-borrows when it needs the VM.

2. **Closure environment capture timing.** Currently, `new_function()` stores the `Environment` directly in `IshFunction`. With shim wrapping, the environment is captured inside the shim closure at declaration time. The timing is the same, but the mechanism changes from struct field to closure capture.

3. **Async functions.** Currently `IshFunction.is_async` controls whether the interpreter spawns a task. With shim-only architecture, async interpreted functions need their shim to handle the spawning. The `is_async` field remains on `IshFunction` as metadata for the caller.

4. **Performance.** Adding an `Rc<dyn Fn>` indirection for every interpreted function call adds one virtual dispatch. This is negligible compared to tree-walking interpretation overhead.

5. **`new_function()` removal.** The current `new_function()` creates an `Interpreted` variant. After this change, the VM's function-declaration handling replaces `new_function()` with shim construction. `new_compiled_function()` becomes the sole factory function.

6. **Yielding classification.** `IshFunction.has_yielding_entry` remains — it's metadata about the function, not an implementation detail. Shims don't change how yielding is classified.

### Proposed Implementation

1. **Change `IshVm` to use `Rc<RefCell<IshVm>>` globally.** Replace every `&mut self` method with a pattern where callers hold an `Rc<RefCell<IshVm>>`, borrow mutably only for the specific mutation, and release before any shim calls. This is a pervasive change to the interpreter, builtins, and shell entry points.

2. **Remove `FunctionImplementation` enum.** Replace with a single `Shim` field on `IshFunction`:
   ```rust
   pub struct IshFunction {
       pub name: Option<String>,
       pub params: Vec<String>,
       pub param_types: Vec<Option<TypeAnnotation>>,
       pub return_type: Option<TypeAnnotation>,
       pub shim: Shim,
       pub is_async: bool,
       pub has_yielding_entry: Option<bool>,
   }
   ```

3. **Remove `closure_env` field from `IshFunction`.** The environment is captured inside the shim.

4. **Modify the VM's function-declaration handling:**
   - Where the VM currently calls `new_function(name, params, ..., body, env, is_async)`, it constructs a shim closure that captures `body`, `env`, and the `Rc<RefCell<IshVm>>`.
   - The shim closure, when called, borrows the VM, creates a child scope from the captured `env`, binds parameters, and calls `exec_statement` on the body.

5. **Remove `new_function()`.** `new_compiled_function()` becomes the sole factory.

6. **Simplify `call_function` in the interpreter:** Currently dispatches on `FunctionImplementation::Interpreted` vs `Compiled`. After this change, all functions are shims — invoke the shim directly. The interpreter still handles arity checking, parameter type auditing, and return type auditing around the shim call.

7. **Verify async function handling.** Ensure async interpreted functions' shims correctly spawn tasks.

### Decisions

All decisions settled (Decisions 9, 11).

---

## Feature 2: Create ish-core Crate and Move Value/Shim to ish-runtime

### ish-core Crate

`TypeAnnotation` is currently defined in `ish-ast` and referenced by `IshFunction` for parameter and return types. Rather than making `ish-runtime` depend on `ish-ast` (which contains the full AST and parser support types), `TypeAnnotation` is extracted into a new `ish-core` crate that both `ish-ast` and `ish-runtime` depend on.

### Move Value and Shim

With the shim-only architecture from Feature 1 complete, `Value` no longer depends on `Environment` or `Statement`. The transitive closure is clean:

- `Value` → `ObjectRef` (gc), `ListRef` (gc), `FunctionRef` → `IshFunction` → `Shim` → `Value` + `RuntimeError`, `FutureRef` → tokio
- `IshFunction` → `TypeAnnotation` (from `ish-core`)

No `ish-ast` dependency. No `Environment` dependency.

### Issues to Watch Out For

1. **GC dependency.** `ish-runtime` must add the `gc` crate. `Value`, `ObjectRef`, `ListRef`, and `FunctionRef` all use `Gc<>`.

2. **Tokio dependency.** `FutureRef` wraps `tokio::task::JoinHandle`. `ish-runtime` must add `tokio`.

3. **Re-export churn.** All `use crate::value::*` and `use crate::error::*` in `ish-vm` must change to `use ish_runtime::*`. Mechanical but large.

4. **ish-core scope.** Initially, `ish-core` contains only `TypeAnnotation` (and any types it depends on). It may grow over time if other shared types emerge, but should not be over-populated preemptively.

5. **ish-ast update.** `ish-ast` must be updated to re-export `TypeAnnotation` from `ish-core` (or consumers update their imports). A re-export is preferred for backward compatibility.

6. **Dependency graph after extraction:**
   ```
   ish-core (TypeAnnotation)
     ↑           ↑
   ish-ast    ish-runtime (Value, Shim, RuntimeError, ErrorCode, IshFunction)
     ↑           ↑
   ish-parser  ish-vm (interpreter, Environment, builtins)
     ↑           ↑
     └─── ish-codegen, ish-stdlib, ish-shell
   ```

### Proposed Implementation

**Phase A: Create ish-core**

1. **Create `proto/ish-core/`** with `Cargo.toml` and `src/lib.rs`.
2. **Move `TypeAnnotation`** (and any types it depends on) from `ish-ast` to `ish-core`.
3. **Add `ish-core` dependency** to `ish-ast/Cargo.toml`.
4. **Re-export `TypeAnnotation`** from `ish-ast` for backward compatibility.
5. **Update workspace** `Cargo.toml` to include `ish-core`.

**Phase B: Move types to ish-runtime**

1. **Add dependencies** to `ish-runtime/Cargo.toml`: `gc` (with derive), `tokio` (workspace features), `ish-core`.

2. **Move source files:**
   - `ish-vm/src/value.rs` → `ish-runtime/src/value.rs` (minus `FunctionImplementation` enum, minus `new_function()`, minus `Environment` references; `TypeAnnotation` imports from `ish-core` instead of `ish-ast`)
   - `ish-vm/src/error.rs` → `ish-runtime/src/error.rs`

3. **Update `ish-runtime/src/lib.rs`:** declare `value` and `error` modules, re-export key types (`Value`, `Shim`, `RuntimeError`, `ErrorCode`, `IshFunction`, `FunctionRef`, `ObjectRef`, `ListRef`, `FutureRef`, `new_compiled_function`, `new_object`, `new_list`, `empty_object`).

4. **Remove `IshValue`** from `ish-runtime/src/lib.rs` (Feature 4, done earlier).

5. **Update `ish-vm`:**
   - Add `ish-runtime` as a dependency.
   - Delete `ish-vm/src/value.rs` and `ish-vm/src/error.rs`.
   - Replace all `use crate::value::*` with `use ish_runtime::*`.
   - Replace all `use crate::error::RuntimeError` with `use ish_runtime::RuntimeError`.
   - Re-export from `ish-vm/src/lib.rs` for backward compatibility if desired.

6. **Update downstream crates** (`ish-codegen`, `ish-stdlib`, `ish-shell`): import shared types from `ish-runtime`. They may still depend on `ish-vm` for the interpreter but not for types.

### Decisions

All decisions settled (Decisions 1, 2, 4, 5, 8, 12).

---

## Feature 3: Error Code Enum

### Issues to Watch Out For

1. **Existing codes are string literals.** The 13 error codes (E001–E013) are scattered across the interpreter, builtins, and environment modules with no central registry.

2. **`RuntimeError::system_error()` constructs `Value::Object`.** This method creates a `Value::Object` with `message` and `code` properties. After the extraction, both `RuntimeError` and `Value` are in `ish-runtime`, so this method can stay intact.

3. **Extensibility.** The enum is closed. New codes require updating `ish-runtime`. This is acceptable — error codes are a core language concern, not something third-party packages should define ad hoc.

4. **Breaking change.** All error creation sites must change from string literals to `ErrorCode` variants.

### Proposed Implementation

1. **Define `ErrorCode` enum** in `ish-runtime/src/error.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    /// E001: Unhandled throw / system error
    UnhandledThrow,
    /// E002: Division by zero / modulo by zero
    DivisionByZero,
    /// E003: Argument count mismatch
    ArgumentCountMismatch,
    /// E004: Type mismatch / undefined operation
    TypeMismatch,
    /// E005: Undefined variable
    UndefinedVariable,
    /// E006: Cannot call non-function
    NotCallable,
    /// E007: Index out of bounds
    IndexOutOfBounds,
    /// E008: I/O error
    IoError,
    /// E009: Null unwrap via `?` operator
    NullUnwrap,
    /// E010: External command / shell error
    ShellError,
    /// E011: Async / concurrency error
    AsyncError,
    /// E012: Cannot await unyielding function
    AwaitUnyielding,
    /// E013: Cannot spawn unyielding function
    SpawnUnyielding,
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::UnhandledThrow => "E001",
            ErrorCode::DivisionByZero => "E002",
            // ... etc
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
```

2. **Update `RuntimeError::system_error()`** to accept `ErrorCode` instead of `&str`:

```rust
pub fn system_error(message: impl Into<String>, code: ErrorCode) -> RuntimeError { ... }
```

3. **Update all call sites** in the interpreter, builtins, and environment to use `ErrorCode` variants.

### Decisions

All decisions settled (Decision 7).

---

## Feature 4: Remove IshValue

### Proposed Implementation

1. Remove the `IshValue` enum, its `impl` block, and its tests from `ish-runtime/src/lib.rs`.
2. Verify `ish-codegen/src/template.rs` does not reference `IshValue` in generated code (it currently does not).
3. Remove any `use ish_runtime::IshValue` imports if they exist in other crates (none currently exist).

### Decisions

All decisions settled (Decision 6).

---

## Feature 5: Cross-Boundary Function Call Testing

The shim-only architecture must be validated with deep nesting tests that exercise the interpreted→compiled→interpreted boundary multiple times.

### Issues to Watch Out For

1. **Borrow conflicts.** The most likely failure mode is a double-borrow panic when a compiled shim calls an interpreted-function shim that borrows the VM while the VM is already borrowed. Testing must specifically exercise this.

2. **Stack depth.** Deep nesting should verify that the Rust call stack doesn't overflow. The shim indirection adds frames.

3. **Closure correctness.** Interpreted functions closed over local variables must correctly resolve those variables when called back from compiled code.

### Proposed Implementation

1. **Add a builtin function** (e.g., `apply` or `call_with`) that takes a function and arguments, then calls the function. This is implemented as a compiled shim, so calling it with an interpreted function produces the interpreted→compiled→interpreted path.

2. **Write acceptance tests** that chain calls through multiple layers:
   ```
   // interpreted function
   let f = fn(x) { x + 1 }
   
   // compiled builtin calls interpreted function
   let result = apply(f, [10])  // → 11
   
   // interpreted function that calls compiled builtin that calls interpreted function
   let g = fn(x) { apply(f, [x]) }
   let result2 = apply(g, [20])  // → 21 (4 boundary crossings)
   
   // deeper nesting
   let h = fn(x) { apply(g, [x]) }
   let result3 = apply(h, [30])  // → 31 (6 boundary crossings)
   ```

3. **Test closures across boundaries:**
   ```
   let base = 100
   let closed = fn(x) { x + base }
   let result = apply(closed, [5])  // → 105
   ```

Cross-boundary error propagation testing is deferred — error propagation across the compiled layer is a behavior of the compiled function, not the interpreter.

### Decisions

All decisions settled (Decision 10).

---

## Implementation Order

The features have dependencies:

1. **Feature 4: Remove IshValue** — independent, do first to simplify the crate.
2. **Feature 1: Shim-only architecture** — refactor `IshVm` to `Rc<RefCell<IshVm>>`, refactor `IshFunction`, remove `FunctionImplementation::Interpreted`, modify interpreter.
3. **Feature 5: Cross-boundary testing** — validate Feature 1 works correctly.
4. **Feature 3: Error code enum** — can be done in parallel with Feature 2, but doing it first means `RuntimeError` moves in its final form.
5. **Feature 2: Create ish-core and move Value/Shim** — the actual extraction, depends on Features 1, 3, and 4 being complete.

---

## Documentation Updates

The following documentation files will need updates:

| File | Update needed |
|------|--------------|
| [docs/architecture/runtime.md](../../architecture/runtime.md) | Rewrite: now contains `Value`, `Shim`, `RuntimeError`, `ErrorCode`, `IshFunction`, `FutureRef`. No longer "minimal." |
| [docs/architecture/vm.md](../../architecture/vm.md) | Remove Value/Error type sections. Update function dispatch: all functions are shims. Describe `Rc<RefCell<IshVm>>` pattern and interpreted-function shim wrapping. |
| [docs/architecture/codegen.md](../../architecture/codegen.md) | Update dependency description — codegen depends on runtime for types. |
| [docs/architecture/overview.md](../../architecture/overview.md) | Update crate map and dependency diagram. Add `ish-core`. `ish-runtime` is now substantial. |
| [AGENTS.md](../../../AGENTS.md) | Update Prototype Crate Map: add `ish-core`, update `ish-runtime` description. |
| [GLOSSARY.md](../../../GLOSSARY.md) | Update "Shim function", "Compiled function", "Function implementation" entries. Remove `Interpreted` variant references. Add "Runtime crate" and "Core crate" entries. |
| [proto/ARCHITECTURE.md](../../../proto/ARCHITECTURE.md) | Update crate descriptions. Add `ish-core`. |
| [docs/errors/INDEX.md](../../errors/INDEX.md) | Update production site references to `ish-runtime::error`. Add `ErrorCode` enum reference. |
| [docs/spec/types.md](../../spec/types.md) | Update reference to `Value` type location. Note `TypeAnnotation` now in `ish-core`. |
| [docs/spec/concurrency.md](../../spec/concurrency.md) | Update references to `FutureRef` location. |

Remember to update `## Referenced by` sections in all modified documentation files.

---

## History Updates

- [x] Create `docs/project/history/2026-04-02-runtime-extraction/` directory
- [x] Add `summary.md` with narrative prose
- [x] Update `docs/project/history/INDEX.md`

---

## Referenced by

- [docs/project/proposals/INDEX.md](INDEX.md)