---
title: "User Guide: Types"
category: user-guide
audience: [human-dev]
status: placeholder
last-verified: 2026-03-19
depends-on: [docs/spec/types.md]
---

# Types

ish has a rich type system that adapts to your needs. In low-assurance mode, types are inferred automatically. In high-assurance mode, you declare types explicitly for stricter checking.

Values carry **entries** — facts recorded by the assurance ledger. Type information is expressed through value entries rather than a separate type system. See [docs/spec/types.md](../spec/types.md) for the full specification.

---

## Primitive Types

Numeric types match Rust: `i8`, `i16`, `i32`, `i64`, `i128`, `u8`–`u128`, `usize`, `f32`, `f64`. Plus `bool` and `char`.

In low-assurance mode, numbers default to `f64`:

```ish
let x = 42      // f64 in low-assurance mode
let y = 3.14    // f64
```

## Objects

```ish
let person = {
    name: "Alice",
    age: 30,
}
```

Object literals are **closed** by default — they have exactly the declared properties. Use `@[Open]` to allow extra properties:

```ish
@[Open]
let config = load_config()    // extra properties allowed
```

Type declarations are **indeterminate** — neither open nor closed until annotated or determined by the active standard.

## Lists

```ish
let numbers = [1, 2, 3, 4, 5]
```

## Union Types

```ish
let value: i32 | String = get_value()
```

## Intersection Types

Combine multiple types — an intersection satisfies all constituent types:

```ish
type Named = { name: String }
type Aged = { age: i32 }
type Person = Named & Aged    // { name: String, age: i32 }
```

## Optional Types

```ish
let x: i32? = maybe_get_number()   // i32 or null
```

## Type Narrowing

Control flow narrows types automatically:

```ish
let x: i32 | String = get_value()
if is_type(x, String) {
    // x is narrowed to String here
}
```

See [docs/spec/assurance-ledger.md](../spec/assurance-ledger.md) for how narrowing works as ledger entry maintenance.

---

## Referenced by

- [docs/user-guide/INDEX.md](INDEX.md)
