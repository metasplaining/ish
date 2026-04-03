---
title: Architecture Overview
category: architecture
audience: [all]
status: draft
last-verified: 2026-04-02
depends-on: [docs/spec/execution.md, docs/spec/modules.md, docs/spec/concurrency.md]
---

# Architecture Overview

High-level architecture of the ish prototype language processor.

---

## Crate Dependency Graph

```
ish-core (standalone — TypeAnnotation, serde)
  ↑           ↑
ish-ast    ish-runtime ── ish-core, gc, tokio
  ↑           ↑
ish-parser  ish-vm ────── ish-ast, ish-runtime, gc, serde_json, tokio
  ↑           ↑
  └─── ish-codegen ─── ish-ast, ish-vm, ish-runtime, libloading, tempfile
  └─── ish-stdlib ──── ish-ast, ish-vm
  └─── ish-shell (binary)
```

`ish-core` contains shared types (primarily `TypeAnnotation`) used by both `ish-ast` and `ish-runtime`.

`ish-runtime` contains the runtime type system (`Value`, `Shim`, `RuntimeError`, `ErrorCode`, `IshFunction`) and is intentionally free of interpreter dependencies so that compiled packages can link against it without pulling in the full VM.

---

## Concurrency Runtime

The prototype uses Tokio for asynchronous execution:

- **ish-vm** depends on `tokio` and runs all user code inside a `LocalSet` with `spawn_local`. This keeps values `!Send`-safe while enabling cooperative multitasking.
- **ish-shell** in interactive mode uses a two-thread architecture: a shell thread (Reedline, parser) and a main thread (Tokio `LocalSet`, VM). See [shell.md](shell.md) for details.
- **Parallel tasks** use `tokio::spawn` on the multi-threaded runtime, restricted to Rust-level parallel shims that cannot access GC-managed values.

The interpreter's `eval` function is `async`, and yield budget checks insert `tokio::task::yield_now().await` at yield-eligible points. See [docs/spec/concurrency.md](../spec/concurrency.md) for the full model.

---

## Crate Summary

| Crate | Purpose | Source |
|-------|---------|--------|
| [ish-core](overview.md) | Shared types (`TypeAnnotation`) used by both AST and runtime | `proto/ish-core/` |
| [ish-ast](ast.md) | AST node types, builder API, display formatting | `proto/ish-ast/` |
| [ish-vm](vm.md) | Tree-walking interpreter, Environment, builtins, reflection | `proto/ish-vm/` |
| [ish-stdlib](stdlib.md) | Self-hosted analyzer, Rust generator, standard library | `proto/ish-stdlib/` |
| [ish-runtime](runtime.md) | Runtime types: Value, Shim, RuntimeError, ErrorCode, IshFunction | `proto/ish-runtime/` |
| [ish-codegen](codegen.md) | Compilation driver: temp Cargo project → `cargo build` → load `.so` | `proto/ish-codegen/` |
| [ish-shell](shell.md) | CLI binary running verification demos | `proto/ish-shell/` |

---

## Key Implementation Patterns

### Pattern: `ref` on GC values

Because `Value` implements `Drop` (via the gc crate), pattern matching must use `ref` to borrow rather than move:

```rust
match value {
    Value::Object(ref obj) => { ... }  // borrow the Gc pointer
    Value::String(ref s) => { ... }    // borrow the Rc<String>
    _ => { ... }
}
```

### Pattern: `ControlFlow::ExprValue`

The interpreter returns the last expression statement's value from `run()` without re-evaluating it. `ControlFlow::ExprValue(value)` carries this through `exec_statement` back to `run()`.

### Pattern: Self-hosted programs via builder API

Each self-hosted component follows the same pattern:

```rust
pub fn register_X(vm: &mut IshVm) {
    let program = build_X();          // Build ish AST via ProgramBuilder
    vm.run(&program).unwrap();        // Execute it → defines functions in global env
}
```

### Pattern: AST-as-values for self-hosting

Self-hosted tools receive AST nodes as ish Objects (via `program_to_value()`), walk them by reading the `"kind"` field with `obj_get()`, and recursively process child nodes. No special AST visitor infrastructure needed.

---

## File Index

```
proto/
├── Cargo.toml                          Workspace: 8 members
├── README.md                           Prototype overview and quick start
├── ish-core/src/
│   └── lib.rs                          TypeAnnotation enum
├── ish-ast/src/
│   ├── lib.rs                          AST types, convenience constructors (8 tests)
│   ├── builder.rs                      ProgramBuilder, BlockBuilder (2 tests)
│   └── display.rs                      fmt::Display for AST (1 test)
├── ish-vm/src/
│   ├── lib.rs                          Module declarations, re-exports from ish-runtime
│   ├── environment.rs                  Lexical scope chain
│   ├── interpreter.rs                  IshVm (Rc<RefCell>), eval, exec, call
│   ├── builtins.rs                     Built-in functions
│   ├── reflection.rs                   AST↔Value conversion, AST factories
│   └── ledger/                         Assurance ledger runtime
├── ish-runtime/src/
│   ├── lib.rs                          Re-exports
│   ├── value.rs                        Value enum, Shim, IshFunction, ObjectRef, ListRef, FunctionRef, FutureRef
│   └── error.rs                        RuntimeError, ErrorCode
├── ish-stdlib/src/
│   ├── lib.rs                          load_all() entry point
│   ├── analyzer.rs                     Self-hosted code analyzer (4 tests)
│   ├── generator.rs                    Self-hosted Rust generator (3 tests)
│   └── stdlib.rs                       abs, max, min, range, etc. (6 tests)
├── ish-codegen/src/
│   ├── lib.rs                          CompilationDriver (2 tests)
│   └── template.rs                     Cargo.toml + lib.rs templates (2 tests)
└── ish-shell/src/
    └── main.rs                         End-to-end verification demos
```

---

## Referenced by

- [docs/architecture/INDEX.md](INDEX.md)
- [docs/INDEX.md](../INDEX.md)
- [docs/spec/concurrency.md](../spec/concurrency.md)
