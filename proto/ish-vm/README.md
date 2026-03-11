# ish-vm

The virtual machine crate. Executes AST programs via tree-walking interpretation.

## Responsibility

- Tree-walking interpreter
- Environment/scope management
- Value representation at runtime
- Builtin functions (print, type-of, len, etc.)
- Reflection subsystem
- Runtime error handling

## Key Files

| File | Purpose |
|------|---------|
| `src/lib.rs` | Module root and public API |
| `src/interpreter.rs` | Tree-walking evaluator |
| `src/environment.rs` | Scope and variable binding |
| `src/value.rs` | Runtime value representation |
| `src/builtins.rs` | Built-in function implementations |
| `src/reflection.rs` | Runtime type introspection |
| `src/error.rs` | Error types |

## Architecture

See [docs/architecture/vm.md](../../docs/architecture/vm.md) for detailed architecture documentation.

## Dependencies

- `ish-ast` — AST node definitions
