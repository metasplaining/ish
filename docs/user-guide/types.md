---
title: "User Guide: Types"
category: user-guide
audience: [human-dev]
status: draft
last-verified: 2026-03-10
depends-on: [docs/spec/types.md]
---

# Types

ish has a rich type system that adapts to your needs. In streamlined mode, types are inferred automatically. In encumbered mode, you declare types explicitly for stricter checking.

For the full specification, see [docs/spec/types.md](../spec/types.md).

---

## Primitive Types

Numeric types match Rust: `i8`, `i16`, `i32`, `i64`, `i128`, `u8`–`u128`, `usize`, `f32`, `f64`. Plus `bool` and `char`.

In streamlined mode, numbers default to `f64`:

```
let x = 42;      // f64 in streamlined mode
let y = 3.14;    // f64
```

## Objects

```
let person = {
    name: "Alice",
    age: 30,
};
```

## Lists

```
let numbers = [1, 2, 3, 4, 5];
```

## Union Types

```
let value: i32 | String = getValue();
```

## Optional Types

```
let x: i32? = maybeGetNumber();   // i32 or null
```

---

## Referenced by

- [docs/user-guide/INDEX.md](INDEX.md)
