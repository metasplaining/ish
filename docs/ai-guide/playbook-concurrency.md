---
title: "AI Playbook: Concurrency"
category: ai-guide
audience: [ai-agent]
status: draft
last-verified: 2026-03-31
depends-on: [docs/spec/concurrency.md, docs/spec/assurance-ledger.md, docs/ai-guide/playbook-low-assurance.md, docs/ai-guide/playbook-high-assurance.md]
---

# Playbook: Concurrency

Use this playbook when generating or modifying ish code that involves async operations, background tasks, concurrent iteration, or yield control.

## When to Use

- Code that calls async standard library functions (I/O, networking)
- Code that uses `spawn`, `await`, or `yield`
- Code that processes collections concurrently
- Code that specifies yield budgets or unyielding blocks

## Assurance-Dependent Behavior

The same concurrent logic looks very different depending on assurance level. **Check the active standard first.**

### Low Assurance (`streamlined`)

- **Do not add** `async fn`, `await`, or `spawn` unless the user requests explicit concurrency control.
- Async stdlib calls are implicitly awaited — write them like synchronous calls.
- Yield is handled automatically by the time-based budget.
- `future_drop` is disabled — dropping futures silently cancels them.

```ish
// Correct low-assurance: no async keywords
let data = file.read("input.txt")
process(data)
```

### High Assurance (`cautious`, `rigorous`)

- **Always** declare functions with `async fn` if they contain `await` or `yield`.
- **Always** `await` or `spawn` async calls explicitly.
- `future_drop` is enabled — every `Future` must be explicitly awaited or cancelled.
- `guaranteed_yield` is enabled — `complex + unyielding` is a discrepancy.

```ish
// Correct high-assurance: explicit annotations
async fn load_data(path: String) -> String {
    let data = await file.read(path)
    return data
}

let result = await load_data("input.txt")
```

## Patterns

### Spawn-and-Collect

Start multiple tasks, then await all results:

```ish
let f1 = spawn fetch(url1)
let f2 = spawn fetch(url2)
let r1 = await f1
let r2 = await f2
```

### Concurrent Iteration

Use `concurrent_map` when processing a collection concurrently:

```ish
let results = concurrent_map(items, (item) => {
    process(item)
})
```

Results are positionally correlated — `results[i]` corresponds to `items[i]`.

### Resource Cleanup with Cancellation

`defer` and `with` blocks run even in cancelled tasks:

```ish
async fn managed_task() {
    with (resource = open_resource()) {
        await long_operation(resource)
    }
    // resource.close() runs even if cancelled
}
```

## Antipatterns

### Adding async keywords at low assurance

```ish
// WRONG at low assurance — unnecessary ceremony
async fn load(path: String) -> String {
    let data = await file.read(path)
    return data
}

// RIGHT at low assurance
fn load(path) {
    file.read(path)
}
```

### Forgetting to await futures at high assurance

```ish
// WRONG at high assurance — future_drop discrepancy
fn run() {
    spawn some_task()
    // Future dropped without await!
}

// RIGHT at high assurance
async fn run() {
    let result = await spawn some_task()
}
```

### Suppressing yield without good reason

```ish
// WRONG — @[unyielding] on complex code causes discrepancy at high assurance
@[unyielding]
fn process_all(items) {
    for item in items {
        complex_operation(item)
    }
}

// RIGHT — let the yield budget handle it, or use yield every
fn process_all(items) {
    for item in items {
        yield every 1000
        complex_operation(item)
    }
}
```

## Key Rules

1. **Check the standard** before deciding whether to add async annotations.
2. **`spawn` does not make the caller yielding** — only `await` and `yield` do.
3. **Errors propagate through `await`** identically to synchronous throws.
4. **No parallel closures** — `tokio::spawn` is Rust-only. ish closures run on the `LocalSet`.
5. **`concurrent_map` is a library function**, not syntax — do not generate `concurrent for` syntax.
6. **`await` and `spawn` require a function call** — `await 42` or `spawn x` are parse errors. Always write `await fn_name(args)` or `spawn fn_name(args)`.
7. **Parallel builtins use implied await at low assurance** — `println("hello")` works without `await`. At high assurance (`await_required`), write `await println("hello")`.
8. **E012/E013 for unyielding functions** — `await` on an explicitly `@[unyielding]` function throws E012. `spawn` on such a function throws E013. Both check *before* calling.

See also: [Low-assurance playbook](playbook-low-assurance.md) | [High-assurance playbook](playbook-high-assurance.md) | [Mixed playbook](playbook-mixed.md)

---

## Referenced by

- [docs/ai-guide/INDEX.md](INDEX.md)
