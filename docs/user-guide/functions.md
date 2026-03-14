---
title: "User Guide: Functions"
category: user-guide
audience: [human-dev]
status: placeholder
last-verified: 2026-03-10
depends-on: [docs/spec/types.md]
---

# Functions

Functions are first-class values in ish. They can be passed as arguments, returned from other functions, and stored in variables.

For the full specification, see [docs/spec/types.md — Function Types](../spec/types.md#function-types).

---

## Declaration

```
fn add(a, b) {
    return a + b;
}
```

With type annotations (optional in low-assurance mode, required in high-assurance mode):

```
fn add(a: i32, b: i32) -> i32 {
    return a + b;
}
```

## Lambdas

```
let double = (x) => { return x * 2; };
```

## Closures

Functions capture variables from their enclosing scope:

```
fn make_counter() {
    let mut count = 0;
    return () => {
        count = count + 1;
        return count;
    };
}
```

---

## Referenced by

- [docs/user-guide/INDEX.md](INDEX.md)
