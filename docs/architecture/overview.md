---
title: Architecture Overview
category: architecture
audience: [all]
status: draft
last-verified: 2026-03-10
depends-on: [docs/spec/execution.md, docs/spec/modules.md]
---

# Architecture Overview

High-level architecture of the ish prototype language processor.

---

## Crate Dependency Graph

```
ish-shell (binary)
├── ish-ast
├── ish-vm ──────── ish-ast, gc, serde_json
├── ish-stdlib ──── ish-ast, ish-vm
└── ish-codegen ─── ish-ast, ish-vm, ish-runtime, libloading, tempfile

ish-runtime (standalone — no ish-* dependencies)
├── serde
└── serde_json
```

`ish-runtime` is intentionally dependency-free (relative to other ish crates) so that compiled `.so` files can link against it without pulling in the full interpreter.

---

## Crate Summary

| Crate | Purpose | Source |
|-------|---------|--------|
| [ish-ast](ast.md) | AST node types, builder API, display formatting | `proto/ish-ast/` |
| [ish-vm](vm.md) | Tree-walking interpreter, GC-managed values, builtins, reflection | `proto/ish-vm/` |
| [ish-stdlib](stdlib.md) | Self-hosted analyzer, Rust generator, standard library | `proto/ish-stdlib/` |
| [ish-runtime](runtime.md) | Minimal FFI value type shared between interpreter and compiled code | `proto/ish-runtime/` |
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
├── Cargo.toml                          Workspace: 6 members
├── README.md                           Prototype overview and quick start
├── ish-ast/src/
│   ├── lib.rs                          AST types, convenience constructors (8 tests)
│   ├── builder.rs                      ProgramBuilder, BlockBuilder (2 tests)
│   └── display.rs                      fmt::Display for AST (1 test)
├── ish-vm/src/
│   ├── lib.rs                          Module declarations
│   ├── value.rs                        Value enum, ObjectRef, ListRef, FunctionRef
│   ├── environment.rs                  Lexical scope chain
│   ├── interpreter.rs                  IshVm, eval, exec, call (8 tests)
│   ├── builtins.rs                     45 built-in functions (6 tests)
│   ├── reflection.rs                   AST↔Value conversion, AST factories (4 tests)
│   └── error.rs                        RuntimeError type
├── ish-stdlib/src/
│   ├── lib.rs                          load_all() entry point
│   ├── analyzer.rs                     Self-hosted code analyzer (4 tests)
│   ├── generator.rs                    Self-hosted Rust generator (3 tests)
│   └── stdlib.rs                       abs, max, min, range, etc. (6 tests)
├── ish-runtime/src/
│   └── lib.rs                          IshValue enum (1 test)
├── ish-codegen/src/
│   ├── lib.rs                          CompilationDriver (2 tests)
│   └── template.rs                     Cargo.toml + lib.rs templates (2 tests)
└── ish-shell/src/
    └── main.rs                         6 end-to-end verification demos
```

---

## Referenced by

- [docs/architecture/INDEX.md](INDEX.md)
- [docs/INDEX.md](../INDEX.md)
