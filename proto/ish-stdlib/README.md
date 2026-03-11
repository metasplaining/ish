# ish-stdlib

The standard library crate. Implements the self-hosted portion of ish's standard library.

## Responsibility

- Standard library definitions (written as AST)
- Code analyzer for AST annotation
- Rust code generator for compilation

## Key Files

| File | Purpose |
|------|---------|
| `src/lib.rs` | Module root and public API |
| `src/stdlib.rs` | Standard library AST definitions |
| `src/analyzer.rs` | AST analysis and annotation |
| `src/generator.rs` | Rust source code generation from AST |

## Architecture

See [docs/architecture/stdlib.md](../../docs/architecture/stdlib.md) for detailed architecture documentation.

## Dependencies

- `ish-ast` — AST node definitions
- `ish-vm` — for interpreter-based execution
