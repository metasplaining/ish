---
title: "Architecture: ish-codegen"
category: architecture
audience: [all]
status: draft
last-verified: 2026-03-10
depends-on: [docs/architecture/overview.md, docs/architecture/runtime.md]
---

# ish-codegen

**Source:** `proto/ish-codegen/src/`

Compiles generated Rust source into dynamically loadable shared libraries.

---

## CompilationDriver

```rust
pub struct CompilationDriver {
    runtime_path: PathBuf,  // absolute path to ish-runtime crate
}
```

**Pipeline:**

1. Create a temp directory via `tempfile::tempdir()`
2. Write a `Cargo.toml` (cdylib crate type, depends on `ish-runtime`)
3. Write `src/lib.rs` with the generated Rust source
4. Run `cargo build --release`
5. Find the `.so`/`.dylib` in `target/release/`
6. Load via `libloading::Library::new()`
7. Look up function symbol and return a callable function pointer

**Methods:**

| Method | Returns | Description |
|--------|---------|-------------|
| `compile(source)` | `(CompiledLibrary, PathBuf)` | Compile and return the library + .so path |
| `compile_function_1(source, name)` | `(CompiledLibrary, fn(i64) → i64)` | Compile and look up a 1-arg function |
| `compile_function_2(source, name)` | `(CompiledLibrary, fn(i64, i64) → i64)` | Compile and look up a 2-arg function |

The `CompiledLibrary` holds both the `libloading::Library` and the `TempDir` — dropping it unloads the library and cleans up the temp directory.

---

## Templates (`template.rs`)

- `cargo_toml(runtime_path)` — generates `Cargo.toml` with `crate-type = ["cdylib"]`
- `lib_rs(source)` — wraps source with `#![allow(...)]` pragmas

---

## Tests

- `lib.rs`: 2 tests
- `template.rs`: 2 tests

---

## Referenced by

- [docs/architecture/INDEX.md](INDEX.md)
- [docs/architecture/overview.md](overview.md)
