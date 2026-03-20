---
title: Error Handling Specification
category: spec
audience: [all]
status: draft
last-verified: 2026-03-19
depends-on: [docs/spec/types.md, docs/spec/assurance-ledger.md, docs/errors/INDEX.md]
---

# Error Handling

This document specifies ish error handling semantics: the error hierarchy, throw/catch mechanisms, ledger integration, and error propagation.

---

## Error Hierarchy

Errors in ish are ordinary objects annotated with error entries managed by the assurance ledger. The hierarchy is defined by entry type inheritance:

| Entry type | Parent | Required properties | Description |
|-----------|--------|---------------------|-------------|
| `Error` | — | `message: String` | Base error entry — marks a value as an error |
| `CodedError` | `Error` | `code: String` | Error with a well-known error code |
| `SystemError` | `CodedError` | — | Interpreter-generated error (not user-created) |

Domain subtypes extend `CodedError` to classify errors by source:

| Domain subtype | Parent | Description |
|---------------|--------|-------------|
| `TypeError` | `CodedError` | Type mismatch or type system violation |
| `ArgumentError` | `CodedError` | Incorrect argument count or type |
| `FileError` | `CodedError` | File system operation failure |
| `FileNotFoundError` | `FileError` | File does not exist |
| `PermissionError` | `FileError` | Permission denied |

Domain subtypes are entry types registered at startup. User code may define additional domain subtypes with `entry type` declarations.

### Error Object Structure

An error object is any object that carries an `Error` entry. The minimum structure:

```ish
let err = @[Error] { message: "something went wrong" }
```

A coded error adds a `code` property:

```ish
let err = @[Error] { message: "file not found", code: "E004" }
```

System errors are created by the interpreter and carry a `SystemError` entry. System errors are coded errors, and carry a well known error code.  Users may created and throw System errors, if they wish:

```ish
throw { code: "E006", message: "I will be recognized by the type system as a TypeError" }
```
---

## Throw

The `throw` statement raises an error, transferring control to the nearest enclosing `catch` clause or propagating to the caller.

```ish
throw { message: "invalid input" }
```

### Throw Audit

When a `throw` statement executes, the ledger audits the thrown value:

1. **Object with `message: String` and no `Error` entry** — the ledger auto-adds an `Error` entry. This allows natural error creation without explicit annotation.
2. **Object with `message: String` and `code: String` and no `CodedError` entry** — the ledger auto-adds a `CodedError` entry (which implies `Error`).
3. **Object without `message: String`** — discrepancy. The throw produces a `SystemError` wrapping the original value.
4. **Non-object value** — discrepancy. The throw produces a `SystemError` wrapping the original value.

The throw audit ensures that all values in the error propagation path carry proper error entries.

---

## Try / Catch / Finally

```ish
try {
    // code that may throw
} catch (e) {
    // handle error
} finally {
    // always runs
}
```

### Catch Clauses

A `catch` clause binds the thrown value to a parameter. Multiple catch clauses may appear:

```ish
try {
    risky_operation()
} catch (e: FileNotFoundError) {
    println("File missing: " + e.message)
} catch (e: FileError) {
    println("File error: " + e.message)
} catch (e) {
    println("Unknown error: " + e.message)
}
```

Catch clauses are evaluated in order. A catch clause matches when the thrown value's entries satisfy the type annotation. An untyped catch matches all errors.

If no catch clause matches, the error propagates to the next enclosing try/catch or to the caller.

### Finally

A `finally` block always executes, whether or not an error was thrown or caught. `return` statements are not permitted in `finally` blocks.

If the `finally` block throws, the new throw replaces the original.

---

## Defer

The `defer` statement schedules cleanup code to run when the enclosing block exits:

```ish
fn process_file(path: String) {
    let f = open(path)
    defer { close(f) }
    // ... work with f ...
}
```

Deferred statements execute in LIFO (last-in, first-out) order. Errors from deferred statements are silently discarded to prevent masking the original error.

`defer` is scoped to the enclosing block, not the function.

---

## Error Propagation

When a thrown error crosses a function boundary without being caught, it propagates to the caller. The caller observes the same error object with the same entries.

### Return Handler

The return handler mechanism configures how uncaught errors behave at function boundaries. This is an implementation detail documented in [docs/architecture/vm.md](../architecture/vm.md). The return handler can:

- Re-throw the error (default in streamlined mode)
- Convert to a result value (encumbered mode)
- Add stack trace context

### Error Union Types

Functions may declare the errors they can throw using union return types:

```ish
fn read_config(path: String) -> Config | FileNotFoundError | PermissionError {
    // ...
}
```

The `@[throws(E)]` annotation is equivalent:

```ish
@[throws(FileNotFoundError | PermissionError)]
fn read_config(path: String) -> Config {
    // ...
}
```

---

## Ledger Integration

### `undeclared_errors` Feature

The `undeclared_errors` feature controls whether functions can throw errors not declared in their signature:

| State | Meaning |
|-------|---------|
| `any` | Functions may throw any error. No checking performed. |
| `typed` | Functions must declare thrown error types. Undeclared throws produce a discrepancy. |
| `none` | Functions must not throw. Any throw produces a discrepancy. |

The `streamlined` standard defaults to `any`. The `rigorous` standard uses `none`.

### Error Entries and Type Checking

When the `types` feature is active, error entries participate in type checking:

- `is_error(value)` — returns `true` if the value carries an `Error` entry
- `error_message(value)` — returns the `message` property (null if not an error)
- `error_code(value)` — returns the `code` property (null if not a `CodedError`)

### Error Codes

Well-known error codes are documented in [docs/errors/INDEX.md](../errors/INDEX.md). Each code maps to a domain error subtype:

| Code | Domain subtype | Description |
|------|---------------|-------------|
| E001 | `Error` | Unhandled throw |
| E002 | `SystemError` | Division by zero |
| E003 | `ArgumentError` | Argument count mismatch |
| E004 | `TypeError` | Type mismatch |
| E005 | `SystemError` | Undefined variable |
| E006 | `TypeError` | Not callable |
| E007 | `SystemError` | Index out of bounds |
| E008 | `FileError` | File I/O error |
| E009 | `TypeError` | Null unwrap |
| E010 | `SystemError` | Shell command error |

---

## Referenced by

- [docs/spec/INDEX.md](INDEX.md)
- [docs/spec/types.md](types.md)
- [docs/spec/assurance-ledger.md](assurance-ledger.md)
- [docs/errors/INDEX.md](../errors/INDEX.md)
- [docs/user-guide/error-handling.md](../user-guide/error-handling.md)
- [docs/ai-guide/patterns.md](../ai-guide/patterns.md)
