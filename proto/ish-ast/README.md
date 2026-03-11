# ish-ast

The abstract syntax tree crate. Defines the core data structures for representing ish programs.

## Responsibility

- AST node types (expressions, statements, declarations)
- Builder API for constructing ASTs programmatically
- Display/formatting for AST pretty-printing

## Key Files

| File | Purpose |
|------|---------|
| `src/lib.rs` | AST node definitions and core types |
| `src/builder.rs` | Fluent API for constructing AST nodes |
| `src/display.rs` | Pretty-printing implementation |

## Architecture

See [docs/architecture/ast.md](../../docs/architecture/ast.md) for detailed architecture documentation.

## Dependencies

None (leaf crate).
