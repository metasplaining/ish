---
title: "User Guide: Functions"
category: user-guide
audience: [human-dev]
status: draft
last-verified: 2026-03-14
depends-on: [docs/spec/types.md, docs/spec/syntax.md]
---

# Functions

Functions are first-class values in ish. They can be passed as arguments, returned from other functions, and stored in variables.

For the full specification, see [docs/spec/types.md — Function Types](../spec/types.md#function-types) and [docs/spec/syntax.md — Functions and Closures](../spec/syntax.md#functions-and-closures).

---

## Declaration

```ish
fn add(a, b) {
    return a + b
}
```

With type annotations (optional in low-assurance mode, required in high-assurance mode):

```ish
fn add(a: i32, b: i32) -> i32 {
    return a + b
}
```

## Default Parameters

```ish
fn connect(host: String, port: i32 = 8080) {
    // ...
}
```

## Lambdas

Expression-body lambdas use implicit return. Block-body lambdas require explicit `return`.

```ish
let double = (x) => x * 2

let process = (x) => {
    let y = transform(x)
    return y
}
```

## Closures

Functions capture variables from their enclosing scope:

```ish
fn make_counter() {
    let mut count = 0
    return () => {
        count = count + 1
        return count
    }
}
```

## Function Types

```ish
type Handler = fn(Request) -> Response
```

---

## Referenced by

- [docs/user-guide/INDEX.md](INDEX.md)
