# ish-shell

The shell binary crate. Entry point that binds all ish crates into a single executable.

## Responsibility

- CLI entry point
- Orchestrating parsing, analysis, interpretation, and compilation

## Key Files

| File | Purpose |
|------|---------|
| `src/main.rs` | Main entry point |

## Architecture

See [docs/architecture/shell.md](../../docs/architecture/shell.md) for detailed architecture documentation.

## Dependencies

- `ish-ast` — AST node definitions
- `ish-vm` — interpreter
- `ish-stdlib` — standard library
- `ish-codegen` — code generation
- `ish-runtime` — runtime support
