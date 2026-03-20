---
title: "User Guide: Error Handling"
category: user-guide
audience: [human-dev]
status: draft
last-verified: 2026-03-19
depends-on: [docs/spec/errors.md, docs/spec/assurance-ledger.md, docs/spec/syntax.md]
---

# Error Handling

ish uses thrown exceptions for error handling. Errors are values — specifically, objects with `Error` entries managed by the assurance ledger — that are thrown with `throw` and caught with `try`/`catch`/`finally`.

---

## Creating Errors

Errors are ordinary objects with a `message` property. When thrown, the ledger automatically adds an `Error` entry:

```ish
throw { message: "something went wrong" }
```

To create a coded error, add a `code` property — the ledger auto-adds a `CodedError` entry:

```ish
throw { message: "file not found", code: "E004" }
```

You can inspect errors with `is_error()`, `error_message()`, and `error_code()`:

```ish
try {
    throw { message: "oops", code: "E001" }
} catch (e) {
    is_error(e)        // true
    error_message(e)   // "oops"
    error_code(e)      // "E001"
}
```

Only objects with a `message: String` property can be thrown. Throwing a non-qualifying value produces a system error.

---

## Throw

The `throw` statement raises an error:

```ish
throw { message: "file not found" }
```

A throw unwinds execution until it reaches a `try`/`catch` block or a function boundary. When a throw escapes a function, it becomes a runtime error that the caller must handle.

---

## Try / Catch / Finally

Use `try`/`catch` to handle thrown errors, and `finally` for cleanup that must always run:

```ish
try {
    let data = read_file("config.json")
} catch (e) {
    println("Error: " + error_message(e))
} finally {
    println("cleanup complete")
}
```

- The `catch` clause binds the thrown value to a variable (`e` above).
- The `finally` block always executes — whether or not an error was thrown, and whether or not it was caught.
- A throw from the `finally` block replaces any in-flight error. A return from `finally` is not permitted.

### Multiple Catch Clauses

Multiple catch clauses can be provided for type-based matching (when the type system supports it):

```ish
try {
    risky_operation()
} catch (e: FileError) {
    // handle file errors
} catch (e: NetworkError) {
    // handle network errors
}
```

Currently in the prototype, the first catch clause always matches.

---

## The `?` Operator

The `?` operator is syntactic sugar for detecting if a function's return value is an error type and throwing it if it is:

```ish
let data = read_file("config.json")?
```

This is equivalent to:

```ish
let _result = read_file("config.json")
if is_error(_result) {
    throw _result
}
let data = _result
```

The `?` operator can be chained:

```ish
let parsed = parse(read_file("config.json")?)
```

---

## With Blocks (Resource Management)

The `with` statement manages resources that need cleanup. Resources are initialized when the block begins and closed when it exits:

```ish
with (f = open_file("data.txt")) {
    let contents = f.read()
}
// f.close() is called automatically here
```

Multiple resources can be managed in a single `with` block:

```ish
with (src = open_file("in.txt"), dest = open_file("out.txt")) {
    dest.write(src.read())
}
// both are closed in reverse order
```

Key behaviors:
- Resources are closed in reverse order of initialization.
- If the body throws, all resources are still closed before the error propagates.
- If initialization of a later resource fails, all earlier resources are closed.
- If `close()` itself fails, the body's error takes precedence.

---

## Defer

The `defer` statement schedules cleanup to run when the enclosing **function** exits:

```ish
fn process() {
    let conn = connect_to_db()
    defer conn.disconnect()

    // use conn...
}
// conn.disconnect() runs when process() returns
```

Defer is function-scoped, not block-scoped. A `defer` inside a conditional or loop accumulates on the enclosing function's defer stack and executes when the function returns — not when the block exits. This allows resources acquired in unpredictable control flow to be released in reverse order at a single, predictable point:

```ish
fn open_all(paths) {
    let files = []
    for p in paths {
        let f = open_file(p)
        defer f.close()
        list_push(files, f)
    }
    // all files are still open here
    return files
}
// all files are closed in reverse order when open_all returns
```

Multiple defers execute in LIFO (last-in, first-out) order:

```ish
fn example() {
    defer println("first")
    defer println("second")
}
// prints: "second", then "first"
```

Deferred statements run regardless of how the function exits — normally, via return, or via throw.

For block-scoped resource cleanup, use a `with` block instead (see above). If you need arbitrary block-scoped cleanup, extract the block into a helper function.

---

## Error Handling Across Assurance Levels

How error handling is configured varies with the assurance level. The `undeclared_errors` feature controls whether functions must declare thrown errors:

- **Streamlined:** Functions can throw without declaring it (`undeclared_errors(any)`). Unhandled throws become runtime errors.
- **Cautious:** Functions should declare thrown errors, but undeclared throws are allowed.
- **Rigorous:** Functions must not throw undeclared errors (`undeclared_errors(none)`). All error paths must be handled.

See [docs/spec/errors.md](../spec/errors.md) for the error hierarchy and [docs/spec/assurance-ledger.md](../spec/assurance-ledger.md) for configuration details.

---

## Referenced by

- [docs/user-guide/INDEX.md](INDEX.md)
