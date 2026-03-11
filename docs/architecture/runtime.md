---
title: "Architecture: ish-runtime"
category: architecture
audience: [all]
status: draft
last-verified: 2026-03-10
depends-on: [docs/architecture/overview.md]
---

# ish-runtime

**Source:** `proto/ish-runtime/src/`

Minimal value type shared between the interpreter and compiled `.so` files.

---

## IshValue

```rust
pub enum IshValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    Null,
}
```

No GC dependency. Compiled functions work with owned values. Currently the FFI boundary uses `i64` directly (the prototype generates `extern "C" fn(i64) -> i64`).

---

## Tests

- `lib.rs`: 1 test

---

## Referenced by

- [docs/architecture/INDEX.md](INDEX.md)
- [docs/architecture/overview.md](overview.md)
- [docs/architecture/codegen.md](codegen.md)
