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

Errors in ish are ordinary objects annotated with error entries managed by the assurance ledger. The only predefined error-related entry type is `@Error`, which requires a `message: String` property. All other error classifications are ordinary ish types defined structurally.

### Entry Type

| Entry type | Required properties | Description |
|-----------|---------------------|-------------|
| `Error` | `message: String` | Marks a value as an error. Added by the throw audit. |

### Structural Error Types

The remaining error hierarchy is defined using ish's structural type system. These are ordinary types, not entry types. They will move to the standard library when the module/package system is complete.

```ish
type CodedError = Error & { code: String }
type TypeError = CodedError & { code: "E004" }
type ArgumentError = CodedError & { code: "E003" }
type FileNotFoundError = CodedError & { code: "E008" }
type PermissionError = CodedError & { code: "EXXX" }
type FileError = FileNotFoundError | PermissionError
type SystemError = TypeError | ArgumentError | FileError
```

`SystemError` is a union defined *over* the domain types — a `FileNotFoundError` is a `SystemError` because the union includes it, not because it inherits from `SystemError`. The interpreter creates system errors as ordinary objects with `message`, `code`, and an `@Error` entry. The type system recognizes them structurally by their `code` value.

User code may define additional error types using the same structural pattern.

### Error Object Structure

An error object is any object that carries an `@Error` entry. The minimum structure:

```ish
let err = { message: "something went wrong" }
throw err  // throw audit auto-adds @Error entry
```

A coded error adds a `code` property. The type system structurally recognizes it as a `CodedError`:

```ish
throw { message: "file not found", code: "E008" }
// Structurally: CodedError (has message + code)
// Structurally: FileNotFoundError (code == "E008")
// Structurally: FileError (FileNotFoundError is in the FileError union)
// Structurally: SystemError (FileError is in the SystemError union)
```

Users may throw errors with well-known codes. The type system recognizes them structurally:

```ish
throw { code: "E004", message: "expected int" }
// Structurally recognized as TypeError, SystemError
```
---

## Throw

The `throw` statement raises an error, transferring control to the nearest enclosing `catch` clause or propagating to the caller.

```ish
throw { message: "invalid input" }
```

### Throw Audit

When a `throw` statement executes, the ledger audits the thrown value:

1. **Object with `message: String`** — the ledger auto-adds an `@Error` entry if not already present. This allows natural error creation without explicit annotation.
2. **Object without `message: String`** — discrepancy. The throw wraps the original value in a system error object: `{ message: "throw audit: thrown value does not qualify as an error", code: "E001", original: <value> }` with an `@Error` entry.
3. **Non-object value** — discrepancy. The throw wraps the value as in rule 2.

The throw audit only adds `@Error` entries. Whether an error has a `code` property (making it structurally a `CodedError`) is observed by the type system, not by the throw audit.

The throw audit ensures that all values in the error propagation path carry `@Error` entries, guaranteeing that `e.message` is always accessible in catch blocks.

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

The `defer` statement schedules cleanup code to run when the enclosing function exits:

```ish
fn process_file(path: String) {
    let f = open(path)
    defer { close(f) }
    // ... work with f ...
}
```

Deferred statements execute in LIFO (last-in, first-out) order. Errors from deferred statements are silently discarded to prevent masking the original error.

`defer` is scoped to the enclosing function. Deferred statements accumulate in a per-function LIFO stack and execute when the function exits — not when the enclosing block exits. See the [defer-scoping proposal](../project/proposals/defer-scoping.md) for the rationale: resources acquired in dynamic control flow (loops, conditionals) are cleaned up in reverse acquisition order at function exit.

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

Well-known error codes are documented in [docs/errors/INDEX.md](../errors/INDEX.md). Each code maps to a domain error subtype. The runtime crate (`ish-runtime`) defines an `ErrorCode` enum with a type-safe variant for each code (e.g., `ErrorCode::DivisionByZero` for E002).

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
| E011 | `ConcurrencyError` | Concurrency error (cancelled task, panicked task, assurance discrepancy, already-awaited future) |
| E012 | `TypeError` | Await type mismatch (`await` applied to a call to an explicitly unyielding function) |
| E013 | `ConcurrencyError` | Spawn type mismatch (`spawn` applied to a call to an explicitly unyielding function) |
| E014 | `TypeError` | Await type mismatch — `await` applied to a non-future value (runtime check) |
| E015 | `ConcurrencyError` | Unyielding annotation violation — function declared `@[unyielding]` contains yielding operations |

### Concurrency Errors

**Concurrency error (E011):** A catch-all for concurrency-related runtime errors. Covers: awaiting a cancelled `Future`, awaiting a panicked task, `future_drop` discrepancy (dropping a future without awaiting it when the feature is enabled), and attempting to await an already-awaited future. Catchable via `try`/`catch`.

**Await type mismatch (E012):** Thrown when `await` is applied to a call to a function with an explicit unyielding entry (`has_yielding_entry: Some(false)`). The error is thrown *before* the function is called. Does not apply to the ambiguous case (functions with no Yielding entry).

**Spawn type mismatch (E013):** Thrown when `spawn` is applied to a call to a function with an explicit unyielding entry. Like E012, the error is thrown before the function is called.

**Unyielding annotation violation (E015):** Thrown at function declaration time when a function annotated `@[unyielding]` is found by the code analyzer to contain yielding operations (an `await`, `yield`, or call to a yielding function). The function is never defined.

See [docs/spec/concurrency.md](concurrency.md) for full cancellation semantics.

---

## Referenced by

- [docs/spec/INDEX.md](INDEX.md)
- [docs/spec/concurrency.md](concurrency.md)
- [docs/spec/types.md](types.md)
- [docs/spec/assurance-ledger.md](assurance-ledger.md)
- [docs/architecture/vm.md](../architecture/vm.md)
- [docs/errors/INDEX.md](../errors/INDEX.md)
- [docs/user-guide/error-handling.md](../user-guide/error-handling.md)
- [docs/ai-guide/patterns.md](../ai-guide/patterns.md)
