# ish-runtime

The runtime crate. Provides the FFI boundary between compiled ish code and the host environment.

## Responsibility

- Runtime support functions for compiled ish
- FFI boundary between generated Rust code and the ish runtime

## Key Files

| File | Purpose |
|------|---------|
| `src/lib.rs` | Runtime support functions |

## Architecture

See [docs/architecture/runtime.md](../../docs/architecture/runtime.md) for detailed architecture documentation.

## Dependencies

- `ish-ast` — AST node definitions
