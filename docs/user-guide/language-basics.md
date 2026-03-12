---
title: Language Basics
category: user-guide
audience: [human-dev]
status: placeholder
last-verified: 2026-03-10
depends-on: [docs/spec/syntax.md, docs/spec/types.md]
---

# Language Basics

> **Note:** ish syntax has not been finalized. Examples below reflect the design intent.

---

## Variables

```
let x = 5;           // immutable
let mut y = 10;      // mutable
y = 20;              // OK — y is mutable
// x = 10;           // ERROR — x is immutable
```

## Expressions

Standard arithmetic, comparison, and logical operators:

```
let sum = a + b;
let bigger = x > y;
let both = a and b;
```

## Control Flow

```
if (condition) {
    // ...
} else {
    // ...
}

while (condition) {
    // ...
}
```

## Functions

```
fn greet(name) {
    println("Hello, " + name + "!");
}

fn add(a, b) {
    return a + b;
}
```

For more detail, see [Functions](functions.md) and the [type system specification](../spec/types.md).

---

## Referenced by

- [docs/user-guide/INDEX.md](INDEX.md)
