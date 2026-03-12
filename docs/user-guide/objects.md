---
title: "User Guide: Objects"
category: user-guide
audience: [human-dev]
status: placeholder
last-verified: 2026-03-10
depends-on: [docs/spec/types.md]
---

# Objects

Objects are the primary structured data type in ish. See [docs/spec/types.md — The Object Type](../spec/types.md#the-object-type) for the full specification.

---

## Creating Objects

```
let person = {
    name: "Alice",
    age: 30,
};
```

## Structural Typing

By default, ish uses structural typing — two objects are compatible if they have the same shape:

```
let a = { x: 1, y: 2 };
let b = { x: 10, y: 20 };
// a and b have the same type: { x: i32, y: i32 }
```

## Nominal Typing

For cases where shape isn't enough, declare types as nominal:

```
nominal type UserId = i64;
nominal type ProductId = i64;
// UserId and ProductId are NOT interchangeable
```

---

## Referenced by

- [docs/user-guide/INDEX.md](INDEX.md)
