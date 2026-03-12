---
title: "User Guide: Error Handling"
category: user-guide
audience: [human-dev]
status: placeholder
last-verified: 2026-03-10
depends-on: [docs/spec/types.md]
---

# Error Handling

Error handling in ish leverages the type system — errors are values, not exceptions.

---

## Result Types

Functions that can fail return a result type that encodes success or failure:

```
fn read_file(path: String) -> String | FileError { ... }
```

The caller must explicitly handle both branches before using the value.

## Propagation

> **TODO**: Error propagation syntax is not yet designed. See [open questions](../project/open-questions.md#error-handling).

## Panic vs. Recoverable Errors

> **TODO**: The distinction between panics and recoverable errors is not yet specified.

---

## Referenced by

- [docs/user-guide/INDEX.md](INDEX.md)
