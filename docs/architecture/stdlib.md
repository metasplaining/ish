---
title: "Architecture: ish-stdlib"
category: architecture
audience: [all]
status: draft
last-verified: 2026-03-10
depends-on: [docs/architecture/overview.md, docs/architecture/vm.md, docs/architecture/ast.md]
---

# ish-stdlib

**Source:** `proto/ish-stdlib/src/`

Self-hosted components — all written as ish programs (ASTs built using the Rust builder API).

---

## Entry Point

```rust
pub fn load_all(vm: &mut IshVm) {
    stdlib::register_stdlib(vm);      // abs, max, min, range, etc.
    analyzer::register_analyzer(vm);  // analyze(), collect_declarations(), etc.
    generator::register_generator(vm); // generate_rust(), generate_expr(), etc.
}
```

Each `register_*` function builds an AST using `ProgramBuilder`, then executes it on the VM via `vm.run()`. This defines the ish functions in the VM's global environment.

---

## Analyzer (`analyzer.rs`)

Ish functions that inspect AST-as-values and report issues:

| Function | Description |
|----------|-------------|
| `collect_declarations(node, declared)` | Walk AST, collect names from var_decl, function_decl, for_each, and function params |
| `collect_references(node, refs)` | Walk AST, collect all Identifier name references |
| `list_contains(lst, item)` | Linear search helper |
| `check_undeclared(declared, referenced)` | Compare declared vs. referenced, return undeclared |
| `check_returns(node)` | Check if a block contains a return statement |
| `is_constant_expr(node)` | Check if an expression is a literal constant |
| `analyze(program_node)` | Main entry: returns `{ warnings: [...], declared_count, reference_count }` |

---

## Generator (`generator.rs`)

Ish functions that produce Rust source code from AST-as-values:

| Function | Description |
|----------|-------------|
| `rust_op(op)` | Map ish operator names to Rust symbols (`"add"` → `"+"`) |
| `generate_expr(node)` | Generate Rust for an expression node |
| `generate_stmt(node, indent)` | Generate Rust for a statement node (with indentation) |
| `generate_block(node, indent)` | Generate Rust for `{ ... }` blocks |
| `generate_rust(node)` | Main entry: handles `function_decl` and `program` nodes |

**Supported constructs:** literals (with Rust suffixes like `_i64`), identifiers, binary/unary ops, function calls, var declarations, assignments, returns, if/else, while loops, blocks.

---

## Standard Library (`stdlib.rs`)

Higher-level functions, also defined as ish programs:

| Function | Description |
|----------|-------------|
| `abs(x)` | Absolute value |
| `max(a, b)` | Larger value |
| `min(a, b)` | Smaller value |
| `range(n)` | `[0, 1, ..., n-1]` |
| `sum(lst)` | Sum of list elements |
| `map(lst, f)` | Apply f to each element |
| `filter(lst, pred)` | Keep elements where pred returns true |
| `assert(cond, msg)` | Print error if false |
| `assert_eq(a, b, msg)` | Check equality |

---

## Tests

- `analyzer.rs`: 4 tests
- `generator.rs`: 3 tests
- `stdlib.rs`: 6 tests

---

## Referenced by

- [docs/architecture/INDEX.md](INDEX.md)
- [docs/architecture/overview.md](overview.md)
