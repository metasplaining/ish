---
title: "Plan: Runtime Extraction"
category: plan
audience: [ai-dev]
status: completed
last-verified: 2026-04-02
depends-on:
  - docs/project/proposals/runtime-extraction.md
  - docs/architecture/runtime.md
  - docs/architecture/vm.md
  - docs/architecture/codegen.md
  - docs/spec/types.md
  - docs/spec/errors.md
  - docs/spec/concurrency.md
---

# Plan: Runtime Extraction

*Derived from [runtime-extraction.md](../proposals/runtime-extraction.md) on 2026-04-02.*

## Overview

Extract the runtime type system from `ish-vm` into `ish-runtime` so compiled ish packages can depend on a standalone crate without pulling in the interpreter. This requires five features executed in order: remove `IshValue`, refactor to shim-only function architecture with `Rc<RefCell<IshVm>>`, validate with cross-boundary tests, add an `ErrorCode` enum, and finally create `ish-core` and move `Value`/`Shim`/`RuntimeError`/`IshFunction` to `ish-runtime`.

## Requirements

### Feature 4 — Remove IshValue

- R4.1: `IshValue` enum no longer exists in `ish-runtime/src/lib.rs`.
- R4.2: No crate references `IshValue`.
- R4.3: All existing tests pass.

### Feature 1 — Shim-Only Function Architecture

- R1.1: `FunctionImplementation` enum no longer exists.
- R1.2: `IshFunction.shim` field is of type `Shim` (`Rc<dyn Fn(&[Value]) -> Result<Value, RuntimeError>>`).
- R1.3: `IshFunction` has no `closure_env` field. Closure environments are captured inside shims.
- R1.4: `new_function()` no longer exists. `new_compiled_function()` is the sole function constructor.
- R1.5: `IshVm` is accessed via `Rc<RefCell<IshVm>>` throughout the interpreter, builtins, and shell entry points. Methods no longer take `&mut self`.
- R1.6: When the VM declares an interpreted function, it wraps the body in a shim closure that captures the `Environment`, `Statement`, and `Rc<RefCell<IshVm>>`.
- R1.7: `call_function` invokes `shim(args)` directly for all functions — no dispatch on implementation variant.
- R1.8: Arity checking, parameter type auditing, and return type auditing still occur around shim invocation.
- R1.9: Async interpreted functions produce shims that correctly spawn tasks.
- R1.10: All existing tests (317 unit + 255 acceptance) pass.

### Feature 5 — Cross-Boundary Function Call Testing

- R5.1: A builtin `apply(fn, args)` exists that calls the given function with the given arguments.
- R5.2: `apply(fn(x) { x + 1 }, [10])` returns `11` (interpreted→compiled→interpreted).
- R5.3: `apply(fn(x) { apply(fn(y) { y + 1 }, [x]) }, [20])` returns `21` (4 boundary crossings).
- R5.4: Nesting at least 3 layers deep (6+ boundary crossings) works correctly.
- R5.5: Closures work across boundaries: `let base = 100; apply(fn(x) { x + base }, [5])` returns `105`.

### Feature 3 — Error Code Enum

- R3.1: `ErrorCode` enum exists in `ish-runtime/src/error.rs` with 13 variants (E001–E013).
- R3.2: `ErrorCode::as_str()` returns the string code (e.g., `"E001"`).
- R3.3: `ErrorCode` implements `Display`.
- R3.4: `RuntimeError::system_error()` accepts `ErrorCode` instead of `&str`.
- R3.5: All error construction sites in interpreter, builtins, and environment use `ErrorCode` variants.
- R3.6: All existing tests pass (error messages unchanged in user-visible output).

### Feature 2 — Create ish-core and Move Types to ish-runtime

- R2.1: `ish-core` crate exists in `proto/ish-core/` with `TypeAnnotation` enum.
- R2.2: `ish-core` is a workspace member.
- R2.3: `ish-ast` depends on `ish-core` and re-exports `TypeAnnotation`.
- R2.4: `ish-runtime` depends on `ish-core`, `gc`, and `tokio`.
- R2.5: `Value`, `Shim`, `ObjectRef`, `ListRef`, `FunctionRef`, `IshFunction`, `FutureRef` are defined in `ish-runtime/src/value.rs`.
- R2.6: `RuntimeError` and `ErrorCode` are defined in `ish-runtime/src/error.rs`.
- R2.7: `new_compiled_function()`, `new_object()`, `new_list()`, `empty_object()` are defined in `ish-runtime`.
- R2.8: `ish-vm` depends on `ish-runtime` and imports types from it. `ish-vm/src/value.rs` and `ish-vm/src/error.rs` no longer exist.
- R2.9: `ish-vm` re-exports key runtime types for backward compatibility.
- R2.10: `ish-codegen` imports shared types from `ish-runtime`.
- R2.11: `ish-stdlib` imports shared types from `ish-runtime`.
- R2.12: All tests pass (317 unit + 255 acceptance).

## Authority Order

1. GLOSSARY.md (new terms)
2. Roadmap (set to "in progress")
3. Specification docs
4. Architecture docs
5. User guide / AI guide
6. Agent documentation (AGENTS.md, skills)
7. Acceptance tests
8. Code (implementation)
9. Unit tests
10. Roadmap (set to "completed")
11. History
12. Index files

## TODO

### Phase 0: Glossary and Roadmap

- [x] 1. **Add glossary terms** — `GLOSSARY.md`
  - Add "Core crate" (`ish-core`): shared types used by both `ish-ast` and `ish-runtime`, primarily `TypeAnnotation`.
  - Add "Runtime crate" (`ish-runtime`): the crate containing `Value`, `Shim`, `RuntimeError`, `ErrorCode`, `IshFunction`, and related types. All compiled ish packages depend on this crate.
  - Update "Compiled function": replace "An `IshFunction` with a `Compiled(Shim)` implementation instead of an `Interpreted(Statement)` body" with "An `IshFunction` whose shim was provided directly (not created by the VM from an interpreted body)."
  - Update "Function implementation": replace the `FunctionImplementation` enum description with "All functions have a shim. The `FunctionImplementation` enum has been eliminated."
  - Update "Shim function": change reference from `docs/architecture/vm.md` to `docs/architecture/runtime.md`.
  - Add "Error code" (`ErrorCode`): a type-safe enum (E001–E013) in `ish-runtime` identifying the category of a `RuntimeError`.

- [x] 2. **Update roadmap** — `docs/project/roadmap.md`
  - Add "Runtime extraction (shim-only architecture, ish-core, type extraction)" under "In Progress".

- [x] 3. **CHECKPOINT: Glossary and roadmap updated.**

### Phase 1: Specification Docs

- [x] 4. **Update spec docs** — `docs/spec/types.md`, `docs/spec/errors.md`, `docs/spec/concurrency.md`
  - Update references to `Value` location (will be `ish-runtime`).
  - Update `FutureRef` references.
  - Add `ErrorCode` enum reference to error spec.

- [x] 5. **CHECKPOINT: Spec docs describe target state.**

### Phase 2: Architecture Docs

- [x] 6. **Update architecture docs** — `docs/architecture/runtime.md`
  - Rewrite to describe: `Value`, `Shim`, `RuntimeError`, `ErrorCode`, `IshFunction`, `FutureRef`, `ObjectRef`, `ListRef`.
  - Describe the purpose: standalone crate for compiled package authors.
  - Update dependency information.

- [x] 7. **Update architecture docs** — `docs/architecture/vm.md`
  - Remove sections describing Value, RuntimeError types (will be in runtime).
  - Add section on shim-only function dispatch and `Rc<RefCell<IshVm>>` pattern.
  - Update crate description.

- [x] 8. **Update architecture overview and codegen** — `docs/architecture/overview.md`, `docs/architecture/codegen.md`
  - Update crate map and dependency diagram to include `ish-core`.
  - Update `ish-runtime` description.
  - Update codegen dependency description.

- [x] 9. **CHECKPOINT: Architecture docs describe target state.**

### Phase 3: Agent Documentation

- [x] 10. **Update AGENTS.md** — `AGENTS.md`
  - Update Prototype Crate Map: add `ish-core`, update `ish-runtime` description.

- [x] 11. **Update proto ARCHITECTURE.md** — `proto/ARCHITECTURE.md`
  - Update crate descriptions to match.

- [x] 12. **Update errors index** — `docs/errors/INDEX.md`
  - Update production site references from `ish-vm::error` to `ish-runtime::error`.
  - Add `ErrorCode` enum reference.

- [x] 13. **CHECKPOINT: Agent docs and error catalog describe target state.**

### Phase 4: Acceptance Tests

- [x] 14. **Write cross-boundary acceptance tests** — `proto/ish-tests/functions/`
  - Test: `apply(fn(x) { x + 1 }, [10])` → `11` (interpreted→compiled→interpreted).
  - Test: `let f = fn(x) { x + 1 }; let g = fn(x) { apply(f, [x]) }; apply(g, [20])` → `21` (4 boundary crossings).
  - Test: 3+ layers deep nesting (6+ boundary crossings).
  - Test: closure capture across boundaries (`let base = 100; apply(fn(x) { x + base }, [5])` → `105`).
  - These tests will fail until the `apply` builtin is implemented in Phase 7.

- [x] 15. **CHECKPOINT: Acceptance tests written (expected to fail until code is complete).**

### Phase 5: Code — Remove IshValue

- [x] 16. **Remove IshValue** — `proto/ish-runtime/src/lib.rs`
  - Delete the `IshValue` enum, its `impl` block, and the `test_conversions` test.
  - Leave `lib.rs` as an empty crate (or with just a comment) for now.

- [x] 17. **Verify** — run `cd proto && cargo build --workspace && cargo test --workspace`

- [x] 18. **CHECKPOINT: IshValue removed, all tests pass.**

### Phase 6: Code — Shim-Only Function Architecture

This is the largest phase. It has two sub-phases: (A) convert `IshVm` to `Rc<RefCell<IshVm>>` and (B) convert functions to shim-only.

#### Phase 6A: Rc<RefCell<IshVm>>

- [x] 19. **Refactor IshVm to Rc<RefCell<IshVm>>** — `proto/ish-vm/src/interpreter.rs`
  - Change all `&mut self` methods to take `vm: &Rc<RefCell<IshVm>>` (or similar pattern).
  - At each call site, borrow mutably only for the specific mutation, then release.
  - The key discipline: never hold a `borrow_mut()` across a function call that might invoke a shim.
  - Update `run()`, `exec_statement()`, `eval_expression()`, `call_function()` and all sub-methods.

- [x] 20. **Update builtins** — `proto/ish-vm/src/builtins.rs`
  - Update `register_builtins()` to work with `Rc<RefCell<IshVm>>`.
  - Builtins that capture VM state (e.g., `print` with channel sender) continue to capture via closures.

- [x] 21. **Update reflection** — `proto/ish-vm/src/reflection.rs`
  - Update all methods that take `&mut IshVm` or `&IshVm`.

- [x] 22. **Update ledger** — `proto/ish-vm/src/ledger/`
  - Update `vm_integration.rs` and any other files that reference `&mut IshVm`.

- [x] 23. **Update shell entry points** — `proto/ish-shell/src/`
  - Update `main.rs` and any REPL loop to create `Rc<RefCell<IshVm>>` and pass it.

- [x] 24. **Update stdlib** — `proto/ish-stdlib/src/`
  - Update any code that creates or interacts with `IshVm`.

- [x] 25. **Verify** — run `cd proto && cargo build --workspace && cargo test --workspace && bash ish-tests/run_all.sh`

- [x] 26. **CHECKPOINT: IshVm is Rc<RefCell<IshVm>> everywhere, all tests pass.**

#### Phase 6B: Shim-Only Functions

- [x] 27. **Remove FunctionImplementation enum** — `proto/ish-vm/src/value.rs`
  - Replace `implementation: FunctionImplementation` with `shim: Shim` on `IshFunction`.
  - Remove `closure_env: Environment` from `IshFunction`.
  - Remove the `FunctionImplementation` enum and its `Debug` impl.
  - Remove `new_function()`.
  - Update `new_compiled_function()` to create the new struct shape.

- [x] 28. **Update interpreter function declarations** — `proto/ish-vm/src/interpreter.rs`
  - Where the interpreter handles `Statement::FunctionDecl` and `Expression::FunctionExpr`, create a shim closure that captures `body` (Statement), `env` (Environment), and `vm` (Rc<RefCell<IshVm>>).
  - The shim closure: borrows VM, creates child scope from captured env, binds parameters, executes body, returns result.

- [x] 29. **Simplify call_function** — `proto/ish-vm/src/interpreter.rs`
  - Remove the `match func.implementation` dispatch.
  - Call `(func.shim)(args)` directly.
  - Keep arity checking, parameter type auditing, return type auditing, defer handling around the call.

- [x] 30. **Update async function handling** — `proto/ish-vm/src/interpreter.rs`
  - Ensure async interpreted functions produce shims that correctly handle task spawning.
  - The `is_async` field on `IshFunction` is metadata; the caller (interpreter) decides whether to spawn.

- [x] 31. **Update builtins for new IshFunction shape** — `proto/ish-vm/src/builtins.rs`
  - All `new_compiled_function()` calls should still work since the factory is updated.
  - Verify no code references `FunctionImplementation` or `closure_env`.

- [x] 32. **Update reflection** — `proto/ish-vm/src/reflection.rs`
  - Update any code that inspects `FunctionImplementation` or `closure_env`.

- [x] 33. **Verify** — run `cd proto && cargo build --workspace && cargo test --workspace && bash ish-tests/run_all.sh`

- [x] 34. **CHECKPOINT: Shim-only architecture complete, all tests pass.**

### Phase 7: Code — Cross-Boundary Builtin

- [x] 35. **Add `apply` builtin** — `proto/ish-vm/src/builtins.rs`
  - `apply(fn, args_list)`: calls `fn` with elements of `args_list` as arguments.
  - Implemented as a compiled shim (it's a builtin), so calling it with an interpreted function creates the interpreted→compiled→interpreted path.

- [x] 36. **Verify** — run `cd proto && cargo test --workspace && bash ish-tests/run_all.sh`
  - Cross-boundary acceptance tests from Phase 4 should now pass.

- [x] 37. **CHECKPOINT: Cross-boundary nesting validated, all tests pass.**

### Phase 8: Code — Error Code Enum

- [x] 38. **Define ErrorCode enum** — `proto/ish-vm/src/error.rs` (still in ish-vm at this point)
  - Add `ErrorCode` enum with 13 variants.
  - Implement `as_str()` and `Display`.

- [x] 39. **Update RuntimeError::system_error()** — `proto/ish-vm/src/error.rs`
  - Change signature from `system_error(msg, &str)` to `system_error(msg, ErrorCode)`.
  - Use `code.as_str()` internally.

- [x] 40. **Update all error sites in interpreter** — `proto/ish-vm/src/interpreter.rs`
  - Replace all `"E001"` through `"E013"` string literals with `ErrorCode` variants.

- [x] 41. **Update all error sites in builtins** — `proto/ish-vm/src/builtins.rs`
  - Replace string literal error codes with `ErrorCode` variants.

- [x] 42. **Update all error sites in environment** — `proto/ish-vm/src/environment.rs`
  - Replace string literal error codes with `ErrorCode` variants.

- [x] 43. **Update error sites in ledger** — `proto/ish-vm/src/ledger/`
  - Check and update any error code string literals.

- [x] 44. **Verify** — run `cd proto && cargo test --workspace && bash ish-tests/run_all.sh`

- [x] 45. **CHECKPOINT: ErrorCode enum complete, all tests pass.**

### Phase 9: Code — Create ish-core and Move Types

#### Phase 9A: Create ish-core

- [x] 46. **Create ish-core crate** — `proto/ish-core/`
  - Create `proto/ish-core/Cargo.toml` with `serde` dependency (for derive).
  - Create `proto/ish-core/src/lib.rs` with `TypeAnnotation` enum (copied from `ish-ast`).

- [x] 47. **Add ish-core to workspace** — `proto/Cargo.toml`
  - Add `"ish-core"` to the workspace members list.

- [x] 48. **Update ish-ast** — `proto/ish-ast/Cargo.toml`, `proto/ish-ast/src/lib.rs`
  - Add `ish-core` dependency.
  - Remove `TypeAnnotation` definition from `lib.rs`.
  - Add `pub use ish_core::TypeAnnotation;` re-export.

- [x] 49. **Verify** — run `cd proto && cargo build --workspace && cargo test --workspace`

- [x] 50. **CHECKPOINT: ish-core exists, ish-ast re-exports TypeAnnotation, all tests pass.**

#### Phase 9B: Move Value, Shim, RuntimeError, ErrorCode to ish-runtime

- [x] 51. **Update ish-runtime dependencies** — `proto/ish-runtime/Cargo.toml`
  - Add: `ish-core`, `gc` (with derive feature), `tokio` (workspace).
  - Remove: `serde`, `serde_json` (unless still needed).

- [x] 52. **Move value.rs** — `proto/ish-vm/src/value.rs` → `proto/ish-runtime/src/value.rs`
  - Copy file contents (post shim-only refactor: no `FunctionImplementation`, no `new_function()`, no `Environment`).
  - Update `use ish_ast::TypeAnnotation` → `use ish_core::TypeAnnotation`.
  - Update any `use crate::error::RuntimeError` → `use crate::error::RuntimeError` (same-crate now).
  - Remove any `use crate::environment::Environment` references.

- [x] 53. **Move error.rs** — `proto/ish-vm/src/error.rs` → `proto/ish-runtime/src/error.rs`
  - Copy file contents (includes `RuntimeError` and `ErrorCode`).
  - Update internal references as needed.

- [x] 54. **Update ish-runtime lib.rs** — `proto/ish-runtime/src/lib.rs`
  - Declare `pub mod value;` and `pub mod error;`.
  - Re-export: `Value`, `Shim`, `RuntimeError`, `ErrorCode`, `IshFunction`, `FunctionRef`, `ObjectRef`, `ListRef`, `FutureRef`, `new_compiled_function`, `new_object`, `new_list`, `empty_object`.

- [x] 55. **Update ish-vm** — `proto/ish-vm/Cargo.toml`, `proto/ish-vm/src/`
  - Add `ish-runtime` dependency to `Cargo.toml`.
  - Delete `proto/ish-vm/src/value.rs` and `proto/ish-vm/src/error.rs`.
  - Remove `pub mod value;` and `pub mod error;` from `lib.rs`.
  - Add `pub use ish_runtime::*;` or specific re-exports to `lib.rs`.
  - Update all `use crate::value::*` → `use ish_runtime::value::*` (or `use ish_runtime::*`).
  - Update all `use crate::error::RuntimeError` → `use ish_runtime::RuntimeError`.
  - Update `interpreter.rs`, `builtins.rs`, `reflection.rs`, `environment.rs`, `ledger/` imports.

- [x] 56. **Update ish-codegen** — `proto/ish-codegen/Cargo.toml`, `proto/ish-codegen/src/`
  - Ensure it imports shared types from `ish-runtime` (it already depends on `ish-runtime`).
  - Update any `use ish_vm::` imports for Value/RuntimeError to `use ish_runtime::`.

- [x] 57. **Update ish-stdlib** — `proto/ish-stdlib/Cargo.toml`, `proto/ish-stdlib/src/`
  - Add `ish-runtime` dependency if not present.
  - Update imports as needed.

- [x] 58. **Update ish-shell** — `proto/ish-shell/Cargo.toml`, `proto/ish-shell/src/`
  - Update imports as needed (shell primarily uses ish-vm, which re-exports).

- [x] 59. **Verify** — run `cd proto && cargo build --workspace && cargo test --workspace && bash ish-tests/run_all.sh`

- [x] 60. **CHECKPOINT: Types moved to ish-runtime, all tests pass.**

### Phase 10: Finalize

- [x] 61. **Final verification** — run all tests
  - `cd proto && cargo build --workspace`
  - `cd proto && cargo test --workspace`
  - `cd proto && bash ish-tests/run_all.sh`
  - `cd proto && cargo run -p ish-shell` (smoke test the REPL)

- [x] 62. **Update roadmap** — `docs/project/roadmap.md`
  - Move "Runtime extraction" from "In Progress" to "Completed".

- [x] 63. **Update history** — `docs/project/history/2026-04-02-runtime-extraction/summary.md`
  - Append implementation narrative describing how the plan was executed.

- [x] 64. **Update index files** — `docs/project/plans/INDEX.md`, `docs/architecture/INDEX.md`
  - Add plan entry.
  - Ensure architecture index reflects new/updated docs.

- [x] 65. **CHECKPOINT: Implementation complete.**

## Reference

### Original file locations (for move operations)

| File | Current location | Destination |
|------|-----------------|-------------|
| `TypeAnnotation` enum | `proto/ish-ast/src/lib.rs` lines 91–107 | `proto/ish-core/src/lib.rs` |
| `value.rs` | `proto/ish-vm/src/value.rs` (397 lines) | `proto/ish-runtime/src/value.rs` |
| `error.rs` | `proto/ish-vm/src/error.rs` (47 lines) | `proto/ish-runtime/src/error.rs` |
| `IshValue` | `proto/ish-runtime/src/lib.rs` (60 lines) | deleted |

### TypeAnnotation definition (to copy to ish-core)

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeAnnotation {
    Simple(String),
    List(Box<TypeAnnotation>),
    Object(Vec<(String, TypeAnnotation)>),
    Function {
        params: Vec<TypeAnnotation>,
        ret: Box<TypeAnnotation>,
    },
    Union(Vec<TypeAnnotation>),
    Optional(Box<TypeAnnotation>),
    Intersection(Vec<TypeAnnotation>),
    Tuple(Vec<TypeAnnotation>),
    Generic {
        base: String,
        type_args: Vec<TypeAnnotation>,
    },
}
```

### Error codes (for ErrorCode enum)

| Code | Variant | Meaning |
|------|---------|---------|
| E001 | `UnhandledThrow` | Unhandled throw / system error |
| E002 | `DivisionByZero` | Division by zero / modulo by zero |
| E003 | `ArgumentCountMismatch` | Argument count mismatch |
| E004 | `TypeMismatch` | Type mismatch / undefined operation |
| E005 | `UndefinedVariable` | Undefined variable |
| E006 | `NotCallable` | Cannot call non-function |
| E007 | `IndexOutOfBounds` | Index out of bounds |
| E008 | `IoError` | I/O error (file read/write) |
| E009 | `NullUnwrap` | Tried to unwrap null with `?` |
| E010 | `ShellError` | External command / shell error |
| E011 | `AsyncError` | Async / concurrency error |
| E012 | `AwaitUnyielding` | Cannot await unyielding function |
| E013 | `SpawnUnyielding` | Cannot spawn unyielding function |

### IshFunction target shape (after shim-only refactor)

```rust
#[derive(Clone, Debug, Trace, Finalize)]
pub struct IshFunction {
    pub name: Option<String>,
    #[unsafe_ignore_trace]
    pub params: Vec<String>,
    #[unsafe_ignore_trace]
    pub param_types: Vec<Option<TypeAnnotation>>,
    #[unsafe_ignore_trace]
    pub return_type: Option<TypeAnnotation>,
    #[unsafe_ignore_trace]
    pub shim: Shim,
    #[unsafe_ignore_trace]
    pub is_async: bool,
    #[unsafe_ignore_trace]
    pub has_yielding_entry: Option<bool>,
}
```

### Dependency graph after extraction

```
ish-core (TypeAnnotation, serde)
  ↑           ↑
ish-ast    ish-runtime (Value, Shim, RuntimeError, ErrorCode, IshFunction; gc, tokio)
  ↑           ↑
ish-parser  ish-vm (interpreter, Environment, builtins; Rc<RefCell<IshVm>>)
  ↑           ↑
  └─── ish-codegen, ish-stdlib, ish-shell
```

### Key decisions (from proposal)

1. All functions are shim-based. No `FunctionImplementation` enum.
2. `Rc<RefCell<IshVm>>` globally. Borrow mutably only when needed, release immediately.
3. `Environment` stays in `ish-vm`. Captured inside shim closures.
4. `ErrorCode` enum with 13 variants replaces string literal error codes.
5. `TypeAnnotation` extracted to `ish-core`. `ish-ast` re-exports it.
6. `IshValue` removed from `ish-runtime`.
7. `Shim` stays as a type alias: `Rc<dyn Fn(&[Value]) -> Result<Value, RuntimeError>>`.
