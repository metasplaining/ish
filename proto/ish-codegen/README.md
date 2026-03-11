# ish-codegen

The code generation crate. Compiles ASTs into Rust source via Handlebars templates.

## Responsibility

- Template-based code generation
- AST-to-Rust translation

## Key Files

| File | Purpose |
|------|---------|
| `src/lib.rs` | Module root and public API |
| `src/template.rs` | Handlebars template definitions and rendering |

## Architecture

See [docs/architecture/codegen.md](../../docs/architecture/codegen.md) for detailed architecture documentation.

## Dependencies

- `ish-ast` — AST node definitions
- `handlebars` — template engine
