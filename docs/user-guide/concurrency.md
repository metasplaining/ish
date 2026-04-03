---
title: "User Guide: Concurrency"
category: user-guide
audience: [human-dev]
status: draft
last-verified: 2026-03-31
depends-on: [docs/spec/concurrency.md, docs/spec/assurance-ledger.md, docs/spec/types.md, docs/spec/syntax.md]
---

# Concurrency

ish provides cooperative multitasking — multiple tasks can run concurrently, sharing a single thread by taking turns. The amount of ceremony depends on your assurance level.

For the full specification, see [docs/spec/concurrency.md](../spec/concurrency.md).

---

## Low-Assurance: Transparent Async

At low assurance (the default), you don't need any concurrency keywords. Async operations happen automatically:

```ish
// No async, await, or spawn needed
let data = file.read("input.txt")
let result = process(data)
file.write("output.txt", result)
```

Under the hood, `file.read` and `file.write` are async operations. The runtime automatically awaits them for you. Your code looks synchronous but runs on an efficient async runtime.

You *can* use concurrency keywords at low assurance if you want explicit control:

```ish
// Spawn a background task
let future = spawn file.read("large-file.txt")

// Do other work while it reads
let config = file.read("config.txt")

// Get the result when needed
let data = await future
```

---

## High-Assurance: Explicit Concurrency

At higher assurance (`cautious`, `rigorous`), you must be explicit about async operations:

```ish
@standard[cautious]

// Must declare async functions
async fn fetch_and_process(path: String) -> Result<String> {
    let data = await file.read(path)
    let result = process(data)
    return result
}

// Must explicitly await
let result = await fetch_and_process("input.txt")
```

Key requirements at high assurance:
- Functions that perform async operations must be declared `async fn`
- Every async call must be explicitly `await`ed or `spawn`ed
- Dropping a `Future` without awaiting it is a discrepancy

---

## Spawn and Futures

`spawn` starts an async operation without waiting for it, returning a `Future<T>`:

```ish
let future = spawn some_async_operation()
// The operation is running in the background

// Later, get the result:
let result = await future
```

If a `Future` goes out of scope without being awaited, the background task is cancelled. Cleanup code in `defer` and `with` blocks still runs even during cancellation.

---

## Concurrent Iteration

Process collections concurrently with `concurrent_map` and `concurrent_for_each`:

```ish
// Process all URLs concurrently
let results = concurrent_map(urls, (url) => {
    http.get(url)
})
// results[i] corresponds to urls[i] — either a value or an error

// Fire-and-forget concurrent processing
let outcomes = concurrent_for_each(items, (item) => {
    process(item)
})
```

Results are positionally correlated with inputs — `results[0]` is the result of processing `urls[0]`, whether it succeeded or failed.

---

## Error Handling

Errors propagate naturally through async boundaries:

```ish
try {
    let data = await fetch_data()
} catch (err) {
    // Catches errors from the async operation,
    // just like synchronous try/catch
    println("Failed: " + err.message)
}
```

If a spawned task throws and you await its future, the error is re-thrown at the `await` site:

```ish
let future = spawn risky_operation()
try {
    let result = await future
} catch (err) {
    println("Background task failed: " + err.message)
}
```

If a `Future` is dropped without being awaited and the task threw, the error is logged (not silently lost).

---

## Yield Control

The runtime automatically ensures that long-running tasks yield control to other tasks periodically (time-based yield budget). At higher assurance levels, you get fine-grained control:

### Explicit Yield

```ish
yield  // Give up control to other tasks right now
```

### Yield in Loops

```ish
for item in large_collection {
    yield every 100  // Yield after every 100 iterations
    process(item)
}
```

### Custom Yield Budget

```ish
@[yield_budget(500us)]
fn time_sensitive() {
    // Yields after 500 microseconds instead of the default ~1ms
}
```

### Suppress Yielding

```ish
@[unyielding]
{
    // This block runs without yielding, even at yield-eligible points
    critical_section()
}
```

At high assurance with `guaranteed_yield` enabled, marking a function as both `complex` and `unyielding` is a discrepancy — complex operations should yield.

---

## Assurance Feature Summary

| Feature | Low (`streamlined`) | Medium (`cautious`) | High (`rigorous`) |
|---------|-------------------|--------------------|--------------------|
| `async_annotation` | Optional (auto-inferred) | Required | Required |
| `await_required` | Optional (auto-await) | Required | Required |
| `guaranteed_yield` | Disabled | Enabled | Enabled |
| `yield_control` | Time-based only | Time-based + statement-count | Time-based + statement-count |
| `future_drop` | Disabled | Enabled | Enabled |

---

## Referenced by

- [docs/user-guide/INDEX.md](INDEX.md)
