---
title: "AI Guide: Antipatterns"
category: ai-guide
audience: [ai-agent]
status: placeholder
last-verified: 2026-03-19
depends-on: [docs/spec/assurance-ledger.md, docs/spec/types.md, docs/spec/errors.md]
---

# Antipatterns

Things AI agents should **not** do when generating ish code.

## 1. Mixing Modes Without a Standard

**Wrong**: Adding type annotations to some variables but not others, without applying a standard.

```ish
// BAD — inconsistent assurance, no standard applied
let x: i32 = 42
let y = "hello"
let z: bool = true
```

**Right**: Either fully low-assurance or fully high-assurance, with a standard applied to make the choice explicit.

## 2. Applying Other Languages' Idioms

**Wrong**: Using Java-style class hierarchies, Python-style duck typing assumptions, or Rust-style lifetime annotations. ish is its own language.

**Right**: Use ish's own abstractions — structural typing, assurance levels, the assurance ledger.

## 3. Inventing Syntax

**Wrong**: Using syntax that looks reasonable but isn't specified. ish's syntax is not yet fully designed — don't guess.

**Right**: Check [docs/spec/syntax.md](../spec/syntax.md) and existing examples. When unsure, ask the user or note the uncertainty.

## 4. Over-Assuring

**Wrong**: Adding types, invariants, and constraints to code the user wanted low-assurance.

**Right**: Match the user's requested assurance level. When unspecified, default to low-assurance.

## 5. Ignoring Assurance Boundaries

**Wrong**: Treating the boundary between low-assurance and high-assurance code as invisible.

**Right**: Explicitly handle type mismatches at boundaries per [assurance-ledger.md](../spec/assurance-ledger.md).

## 6. Assuming Runtime Behavior

**Wrong**: Assuming specific garbage collection, threading model, or memory layout.

**Right**: Check [memory.md](../spec/memory.md) and [execution.md](../spec/execution.md) for what is specified.

## 7. Using Constructor Functions for Errors

**Wrong**: Calling a constructor like `new_error()` to create errors.

```ish
// BAD — new_error() does not exist
let e = new_error("something failed")
throw e
```

**Right**: Throw an object literal with a `message` property. The ledger auto-adds the `Error` entry.

```ish
throw { message: "something failed" }
```

## 8. Throwing Without a Message

**Wrong**: Throwing an object without a `message: String` property.

```ish
// BAD — no message property, throw audit raises a discrepancy
throw { code: "E001" }
```

**Right**: Always include `message: String`. Add `code: String` for coded errors.

```ish
throw { message: "division by zero", code: "E001" }
```

## 9. Await/Spawn on Non-Call Expressions

**Wrong**: Using `await` or `spawn` on a non-function-call expression.

```ish
// BAD — parse error (produces Incomplete node)
let v = await 42
let x = spawn "hello"
```

**Right**: `await` and `spawn` require a function call.

```ish
let v = await some_function()
let x = spawn some_function()
```

## 10. Await/Spawn on Unyielding Functions

**Wrong**: Using `await` or `spawn` on an explicitly unyielding function.

```ish
// BAD — E012 (await) or E013 (spawn)
@[unyielding]
fn pure() { return 5 }

let v = await pure()   // E012
let x = spawn pure()   // E013
```

**Right**: Only `await`/`spawn` functions that are yielding or ambiguous.

```ish
fn maybe_yields() { return do_work() }
let v = await maybe_yields()   // OK — ambiguous, passes through if non-Future
```

---

## Referenced by

- [docs/ai-guide/INDEX.md](INDEX.md)
- [docs/ai-guide/orientation.md](orientation.md)
