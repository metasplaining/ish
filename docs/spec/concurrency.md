---
title: Concurrency
category: spec
audience: [all]
status: draft
last-verified: 2026-03-31
depends-on: [docs/spec/assurance-ledger.md, docs/spec/types.md, docs/spec/syntax.md, docs/spec/execution.md, docs/spec/memory.md, docs/spec/modules.md, docs/spec/errors.md]
---

# Concurrency

ish provides cooperative multitasking via `async`/`await` on a Tokio runtime, with true parallelism available only through Rust-implemented standard library functions. This document specifies the concurrency model, runtime architecture, yield mechanism, parallelism model, data transfer abstractions, I/O types, and the shell/VM execution architecture.

All design decisions are documented in the [concurrency proposal](../project/proposals/concurrency.md).

---

## Cooperative Multitasking

### Runtime Architecture

All ish user code runs inside a Tokio `LocalSet` on the main thread. Tasks are spawned with `spawn_local`, which does not require the `Send` trait. This means:

- GC-managed heap objects can be shared freely between concurrent tasks.
- No synchronization overhead for memory access between ish tasks.
- The single-threaded cooperative model avoids data races by construction.
- The tree-walking interpreter yields at `.await` points without fundamental architectural changes.

```
┌─────────────────────────────────────────────────┐
│ Tokio Runtime (multi-threaded)                   │
│                                                  │
│  ┌──────────────────────────────────┐            │
│  │ LocalSet (main thread)           │            │
│  │                                  │            │
│  │  ┌────────┐ ┌────────┐          │            │
│  │  │ Task 1 │ │ Task 2 │  ...     │ ← ish code │
│  │  │(spawn_ │ │(spawn_ │          │   (!Send)  │
│  │  │ local) │ │ local) │          │            │
│  │  └────────┘ └────────┘          │            │
│  └──────────────────────────────────┘            │
│                                                  │
│  ┌──────────┐ ┌──────────┐                       │
│  │ Parallel │ │ Parallel │  ...     ← Rust libs  │
│  │ Task A   │ │ Task B   │            (Send)     │
│  │ (spawn)  │ │ (spawn)  │                       │
│  └──────────┘ └──────────┘                       │
└─────────────────────────────────────────────────┘
```

### Keywords and Types

| Concept | ish keyword/type | Description |
|---------|-----------------|-------------|
| Async function declaration | `async fn` | Declares a function that can yield |
| Suspend until complete | `await` | Suspends caller until the awaited future resolves |
| Start without waiting | `spawn` | Starts an async operation, returns a `Future` |
| Future handle | `Future<T>` | A value representing an eventual result of type `T` |
| Cooperative suspension | `yield` | Explicitly gives up control to the scheduler |

### Grammar Restrictions on `await` and `spawn`

`await` and `spawn` syntactically require a function call as their operand:

```pest
await_op ~ call_expr    // not await_op ~ unary
spawn_op ~ call_expr    // not spawn_op ~ unary
```

Expressions like `await 42` or `spawn "hello"` are not valid `Await`/`Spawn` nodes — the parser-matches-everything philosophy encodes them as `Incomplete` AST nodes. The AST `Await` and `Spawn` nodes contain a callee and arguments (mirroring `FunctionCall`), not a generic expression.

### Yielding Classification and Validation

Before executing the call, the interpreter checks the callee's yielding classification:

| Callee classification | `await` behavior | `spawn` behavior |
|----------------------|------------------|------------------|
| Explicitly yielding (`has_yielding_entry: Some(true)`) | Normal await — call, then await the resulting `Future` | Normal spawn — spawn a task that calls the function |
| Explicitly unyielding (`has_yielding_entry: Some(false)`) | **E012** — thrown before calling | **E013** — thrown before calling |
| Ambiguous (no Yielding entry, `has_yielding_entry: None`) | Call proceeds; if result is `Future`, await it; if non-`Future`, pass through | Call proceeds; spawns a task that calls the function |

The ambiguous case exists because the prototype does not yet categorize all functions as yielding or unyielding at declaration time. Functions without explicit annotations (`async` or `@[unyielding]`) have no Yielding entry. See open question on function yielding categorization in [docs/project/open-questions.md](../project/open-questions.md).

### Future Identity Equality

Two `Future` values are equal if and only if they reference the same underlying allocation (`Rc::ptr_eq`). This is identity equality, not value equality:

- `f == f` → `true`
- `let g = f; f == g` → `true` (same `Rc` via clone)
- `spawn work() == spawn work()` → `false` (different allocations)

### Implied Await

At low assurance (when the `await_required` feature is not active), calling a function that returns a `Future` without explicit `await` or `spawn` triggers an **implied await**: the interpreter immediately awaits the future and returns the resolved value. This makes parallel builtins like `println` backward-compatible — `println("hello")` works without `await`.

At high assurance (when `await_required` is `required`), no implied await occurs. The `Future` is returned as-is, and the unawaited-future audit detects it.

The only way to obtain a `Future` value (rather than the resolved value) is via `spawn`.

### Low-Assurance Behavior

At low assurance (`streamlined` standard), concurrency is transparent:

- All code is treated as `simple + unyielding` by default.
- When an ish function calls an async standard library function (e.g., file I/O), the call implicitly awaits — the developer does not need to write `await`.
- The runtime automatically upgrades the calling function to `yielding` at execution time.
- The `async`, `await`, `spawn`, and `yield` keywords are available but not required.

```ish
let data = file.read("input.txt")
let result = process(data)
file.write("output.txt", result)
```

The `file.read` and `file.write` calls are async under the hood (Tokio I/O), but the developer sees synchronous-looking code. The runtime awaits each call automatically.

### High-Assurance Behavior

At higher assurance levels (`cautious`, `rigorous`), the developer must be explicit:

```ish
// Explicit async function declaration
async fn fetch_and_process(path: String) -> Result<String> {
    let data = await file.read(path)
    let result = process(data)
    return result
}

// Caller must explicitly await or spawn
let result = await fetch_and_process("input.txt")

// Or spawn to get a future
let future = spawn fetch_and_process("input.txt")
// ... do other work ...
let result = await future
```

### Assurance Features

| Feature | `streamlined` | `cautious` | `rigorous` |
|---------|--------------|------------|------------|
| `async_annotation` | optional (auto-inferred) | required | required |
| `await_required` | optional (default: await) | required | required |
| `guaranteed_yield` | disabled | enabled | enabled |
| `future_drop` | disabled | enabled | enabled |

See [docs/spec/assurance-ledger.md](assurance-ledger.md) for feature state definitions.

---

## Cancellation

### Future Drop

When a `Future` value is dropped (goes out of scope without being awaited), the underlying `spawn_local` task is cancelled via `JoinHandle::abort()`.

Cancellation semantics:

- A cancelled task stops at its next `.await` point.
- `defer` blocks in the cancelled task still execute (cleanup is guaranteed).
- `with` block resources in the cancelled task are still closed.
- Awaiting a cancelled future returns a cancellation error, catchable via `try`/`catch`.

### Error Logging

If a spawned task throws and the `Future` is dropped without being awaited, the error is logged (not silently swallowed).

### `future_drop` Assurance Feature

The `future_drop` feature controls whether dropping a future is flagged:

| `future_drop` state | Behavior |
|---------------------|----------|
| disabled (`streamlined`) | Dropped futures are silently cancelled. No discrepancy. |
| enabled (`cautious`, `rigorous`) | Dropping a future without awaiting it is a discrepancy. The developer must explicitly cancel or await. |

Discrepancies produce runtime errors in the assurance ledger; there is no separate "discrepancy without error" state.

---

## Error Propagation

Errors propagate naturally through `await`:

- If an awaited function throws, the error propagates through the caller's `try`/`catch` chain, identical to synchronous error propagation.
- If a spawned function throws, the error is captured in the `Future`. When the future is awaited, the error is re-thrown at the await site.
- If a `Future` is dropped without being awaited and the task threw an error, the error is logged.

See [docs/spec/errors.md](errors.md) for the cancellation error type.

---

## Concurrent Iteration

Concurrent iteration is provided as standard library functions, not as syntax.

### `concurrent_map`

```ish
// Signature:
// async fn concurrent_map<T, U>(items: [T], fn map_fn(T) -> U) -> U[]
```

Returns an array positionally correlated with the input. Each element contains either a successful result or an error from the corresponding input item:

```ish
let results = concurrent_map(urls, (url) => {
    http.get(url)  // may throw
})
// results[i] is either a Response or an Error, corresponding to urls[i]
for i in 0..results.len() {
    match results[i] {
        response: Response => handle_success(urls[i], response)
        err: Error => handle_failure(urls[i], err)
    }
}
```

The function spawns each iteration as a `spawn_local` task on the `LocalSet`, waits for all to complete, and collects results. Since all tasks run on the same `LocalSet` thread, closures can freely access the enclosing scope's GC-managed data.

When `map_fn` can throw, `U` is a union type (e.g. `ValidResult|Error`). Errors are returned inline — there is no separate `(results, errors)` tuple. This preserves positional correspondence between inputs and outputs.

### `concurrent_for_each`

```ish
// Signature:
// async fn concurrent_for_each<T>(items: [T], fn each_fn(T) -> Void) -> (Void|Error)[]
```

The side-effect variant. Results indicate success (`Void`) or failure (`Error`) at each position.

---

## Ledger Entry Types

Two orthogonal dimensions describe the concurrency properties of code:

### Complexity

```ish
entry type Complexity {
    applies_to: [function, block],
    values: [simple, complex],
}
```

- `simple` — completes quickly; no yield points needed within the block.
- `complex` — may take a long time; yield points should be inserted.

The exact threshold between simple and complex is a runtime configuration, not a language-level guarantee.

**Inference:** A function calling any `complex` function is itself `complex`.

### Yielding

```ish
entry type Yielding {
    applies_to: [function, block],
    values: [yielding, unyielding],
}
```

- `yielding` — the block can suspend execution cooperatively.
- `unyielding` — the block never suspends.

**Inference rules:**

- A function containing an `await` or `yield` expression is `yielding`.
- A function calling any `yielding` function (with `await`) is itself `yielding`.
- Calling `spawn` does **not** make the caller `yielding` — `spawn` returns a `Future` immediately without suspending.
- A function that calls `spawn` is not `yielding` merely from the spawn. However, if it later awaits the resulting future, that `await` makes it `yielding`.

### Parallel

```ish
entry type Parallel {
    applies_to: [function],
    implies: [@[yielding]],
}
```

The `Parallel` entry marks functions implemented via parallel shims — Rust code that runs on Tokio's thread pool. A parallel function is always `yielding` from the caller's perspective because calling it involves an async boundary.

---

## Guaranteed Yield Mechanism

The yield mechanism is a hybrid: time-based yielding by default, with explicit overrides available at higher assurance levels.

### Time-Based Yielding

The runtime maintains a yield budget (configurable time quantum, default ~1ms). At each **yield-eligible point**, the runtime checks whether the budget is exhausted. If exhausted, the runtime inserts a `tokio::task::yield_now().await`, returning control to the Tokio scheduler. The budget is reset after each yield.

**Yield-eligible points (documented yield points):**

1. Every loop back-edge (`for`, `while`)
2. Every function call (before the call)
3. Every explicit `yield` statement

At low assurance, only the time-based mechanism is active. The developer does not need to think about yielding.

### Explicit Yield Control (High Assurance)

At higher assurance levels, the developer can control yield behavior precisely:

```ish
// Time-based — runtime handles it (default at all levels)
for item in items {
    process(item)
}

// Statement-count-based — yield every N iterations
for item in items yield every 500 {
    process(item)
}

// Time-based with custom threshold
@[yield_budget(500us)]
for item in items {
    process(item)
}

// Suppress yielding (marks the block as unyielding)
@[unyielding]
for item in items {
    process(item)
}
```

### Guaranteed Yield as a Discrepancy

When the `guaranteed_yield` feature is enabled (`cautious` and `rigorous` standards), a `complex + unyielding` block is a discrepancy. The developer must either:
- Accept automatic time-based yielding (the block becomes `complex + yielding`)
- Add explicit yield annotations
- Mark the block `@[unyielding]` with justification (an escape hatch that may require review)

### Assurance Features

| Feature | `streamlined` | `cautious` | `rigorous` |
|---------|--------------|------------|------------|
| `guaranteed_yield` | disabled (time-based active but no discrepancies) | enabled (complex + unyielding = discrepancy) | enabled |
| `yield_control` | time-based only | time-based + statement-count | time-based + statement-count |

---

## Parallelism

Parallelism in ish is not a language-level feature — it is a runtime capability exposed through the standard library and the shim system. ish developers consume parallel functions; they do not write them.

### Architecture

- **ish tasks** (cooperative): Run on the `LocalSet`. Share the GC heap. Scheduled cooperatively via `spawn_local`. No `Send` requirement.
- **Parallel tasks** (true parallelism): Run on Tokio's thread pool via `tokio::spawn`. Implemented in Rust. Require `Send`. Cannot access the GC heap directly.

### Parallel Shims

Parallel builtins (`print`, `println`, `read_file`, `write_file`) are compiled functions whose shims internally spawn work on the Tokio thread pool or on a `spawn_local` bridge. A parallel shim is synchronous — it marshals arguments into `Send`-safe form, spawns the work, wraps the `JoinHandle` in a `FutureRef`, and returns `Value::Future` immediately.

```
Unyielding shim:  Fn(&[Value]) -> Result<Value, RuntimeError>         // returns plain Value
Parallel shim:    Fn(&[Value]) -> Result<Value, RuntimeError>         // returns Value::Future
```

Both use the same `Shim` type — the distinction is behavioral, not structural. See [docs/architecture/vm.md](../architecture/vm.md) for the `FunctionImplementation` enum.

When ish code calls a parallel function:
1. The shim marshals ish `Value` arguments into native/`Send`-safe types.
2. The shim spawns work via `spawn_blocking` (for blocking I/O) or `spawn_local` (for async I/O).
3. A `spawn_local` bridge converts native results back to `Value` when the work completes.
4. The shim wraps the `JoinHandle` in a `FutureRef` and returns `Value::Future`.
5. The interpreter handles the future: implied await (low assurance), explicit await, or spawn.

Parallel builtin futures do not resolve until the I/O operation is actually complete. For interactive mode (shell with `ExternalPrinter`), this requires an acknowledgment signal back from the shell after output is written.

### No User-Defined Parallel Functions

No parallel standard library function accepts ish closures. Accepting closures would pull all the parallelism complexity (thread safety, `Send`/`Sync`, data race prevention) back into the language. If a developer needs true CPU-bound parallelism, they write a Rust library with a parallel shim.

### Why No User-Defined Parallel Functions

Allowing ish developers to write parallel functions would require:
1. Ensuring all captured values are `Send` — this leaks Rust's ownership model into ish.
2. Thread-safe GC — the GC would need synchronization for cross-thread access.
3. Data race prevention — shared mutable state across threads.

By restricting parallelism to Rust-implemented shims, these problems become the Rust library author's responsibility.

---

## Data Transfer Abstractions

Data transfer between the `LocalSet` thread and parallel tasks requires careful handling. The design rests on five core abstractions: `ByteBuffer`, `Reader`, `Writer`, `Stream<T>`, and `StreamWriter<T>`.

### ByteBuffer

`ByteBuffer` is a single ish type representing a contiguous sequence of bytes, abstracting over Rust's byte representations:

| Rust type | Characteristics |
|-----------|----------------|
| `&[u8]` / `Vec<u8>` | Heap-allocated, single owner, deep copy on clone |
| `bytes::Bytes` | Reference-counted, immutable, O(1) clone and slice, `Send + Sync` |
| `bytes::BytesMut` | Uniquely-owned, mutable, `Send` only, can freeze to `Bytes` in O(1) |

The runtime selects the optimal Rust representation based on the buffer's `Mutable` ledger entry:

- An **immutable** `ByteBuffer` is backed by `bytes::Bytes`. Supports O(1) cloning and slicing, safely shared across threads.
- A **mutable** `ByteBuffer` (carrying the `@[Mutable]` entry) is backed by `bytes::BytesMut`. Supports efficient append and in-place mutation, but cannot be shared.

Freezing (mutable → immutable) is O(1) via `BytesMut::freeze()`. The reverse (immutable → mutable) requires a copy.

```ish
// Immutable — backed by bytes::Bytes
let data: ByteBuffer = file.read("input.txt")
let slice = data.slice(0, 100)    // O(1), no copy
let copy = data                    // O(1), reference count increment

// Mutable — backed by bytes::BytesMut
let mut buf: ByteBuffer = ByteBuffer.new(4096)
buf.append(data)                   // append bytes
buf.append_string("trailer")      // append from string

// Freeze: mutable → immutable (O(1))
let frozen: ByteBuffer = buf.freeze()
```

**Marshaling for parallel shims:**
- Immutable `ByteBuffer`: clone the `Bytes` handle (O(1), no copy).
- Mutable `ByteBuffer`: freeze (O(1)) or take ownership (move out of the ish heap).
- Complex ish types (objects, lists): copy-in/copy-out semantics in the shim.

### Reader and Writer

`Reader` wraps any Tokio `AsyncRead` source. `Writer` wraps any Tokio `AsyncWrite` sink.

```ish
let reader: Reader = file.open("data.bin")
let chunk: ByteBuffer = reader.read(4096)
let all: ByteBuffer = reader.read_all()
reader.close()

let writer: Writer = file.create("output.bin")
writer.write(data)
writer.flush()
writer.close()
```

### Stream (Unified with Specialization)

`Stream<T>` is a single abstraction with specialized implementations:

- **`Stream<ByteBuffer>`** uses efficient byte-oriented I/O backed by `AsyncRead`/`AsyncWrite` (via `tokio_util::io::ReaderStream`).
- **`Stream<T>`** for other types uses bounded channels (`tokio::sync::mpsc`).

`StreamWriter<T>` is the write half of a stream — the producer side.

Stream combinators work uniformly on all streams:

| Combinator | Tokio mapping |
|-----------|--------------|
| `.lines()` | `FramedRead` with `LinesCodec` |
| `.decode(codec)` | `FramedRead` with the codec's `Decoder` |
| `.filter(predicate)` | `StreamExt::filter()` |
| `.map(transform)` | `StreamExt::map()` |
| `.take(n)` | `StreamExt::take()` |
| `.zip(other)` | `StreamExt::zip()` |

```ish
let stream: Stream<ByteBuffer> = file.stream("large.csv")
let lines: Stream<String> = stream.lines()
let records: Stream<Record> = stream.decode(csv_codec)
let valid_records = records.filter((r) => r.valid)
```

---

## I/O Types

> **Scope note:** The full I/O library described below is future work. For the concurrent implementation, only `println` needs to be implemented — it becomes async internally but its ish-level interface and behavior are unchanged. All other I/O types are deferred.

### File I/O

```ish
// Convenience — whole-file operations
let data: ByteBuffer = file.read("input.txt")
let text: String = file.read_string("input.txt")
file.write("output.txt", data)

// Streaming
let stream: Stream<ByteBuffer> = file.stream("large.csv")

// Reader/Writer
let reader: Reader = file.open("data.bin")
let writer: Writer = file.create("output.bin")
```

### Standard I/O

```ish
// Convenience
let line: String = io.read_line()
io.print("hello")
io.println("hello")
io.eprint("warning")

// Streaming
let lines: Stream<String> = io.stdin().stream().lines()

// Reader/Writer
let stdin: Reader = io.stdin()
let stdout: Writer = io.stdout()
let stderr: Writer = io.stderr()
```

### Network I/O (TCP)

```ish
let conn = tcp.connect("host:8080")
let reader: Reader = conn.reader()
let writer: Writer = conn.writer()

let incoming: Stream<ByteBuffer> = conn.stream()
let messages: Stream<Message> = conn.stream().decode(message_codec)
```

### Network I/O (UDP)

```ish
let socket = udp.bind("0.0.0.0:9000")
socket.send_to("host:9001", data)
let (data, addr) = socket.recv_from()
let datagrams: Stream<{data: ByteBuffer, addr: String}> = socket.stream()
```

### Tokio I/O Summary

| Source | Tokio Type | ish Convenience | ish Stream | Zero-Copy |
|--------|-----------|----------------|------------|-----------|
| stdin | `tokio::io::Stdin` | `io.read_line()` | `io.stdin().stream()` | No |
| stdout | `tokio::io::Stdout` | `io.print()`, `io.println()` | — | No |
| stderr | `tokio::io::Stderr` | `io.eprint()` | — | No |
| File | `tokio::fs::File` | `file.read()`, `file.write()` | `file.stream()` | No |
| TCP | `tokio::net::TcpStream` | — | `conn.stream()` | Yes (`Bytes`) |
| UDP | `tokio::net::UdpSocket` | `socket.send_to()`, `socket.recv_from()` | `socket.stream()` | Yes (`Bytes`) |

### `println` Becomes Async

`println` and `print` are parallel builtins — compiled functions whose shims return `Value::Future`:

1. On the `LocalSet` thread, `println` writes to stdout via the Reedline `ExternalPrinter` (interactive mode) or via `spawn_blocking` to OS stdout (non-interactive mode), making it a yielding operation.
2. The ish-level interface does not change — `println("hello")` still prints "hello" followed by a newline. At low assurance, implied await resolves the future automatically.
3. At high assurance (`await_required` is `required`), the developer must write `await println("hello")` or `spawn println("hello")`.
4. In interactive mode, `println` output from background tasks is routed through the `ExternalPrinter`, so it appears cleanly without corrupting the prompt.
5. The future does not resolve until the I/O operation is actually complete (see Parallel Shims above).

---

## Blocking (Eliminated)

The `blocking` dimension is eliminated from the ish language:

- There is no way to define a blocking function in ish. All ish functions run on the `LocalSet` and use Tokio async I/O.
- Standard library functions that perform I/O use Tokio's async implementations (`tokio::fs`, `tokio::net`, `tokio::process`).
- If a Rust library needs synchronous I/O, it wraps it in a parallel shim (which runs on the Tokio thread pool). From ish's perspective, this is a parallel function, not a blocking one.
- The `Parallel` entry type covers the relevant case.

If ish eventually supports calling arbitrary C/Rust functions via FFI, blocking may need to be reintroduced. See [docs/project/open-questions.md](../project/open-questions.md).

---

## Shell and Execution Architecture

### Two-Thread Model (Interactive Mode)

In interactive mode, there are two threads:

```
┌─ Shell Thread ──────────────────────────────────────┐
│ Reedline read_line() → parse input → submit AST     │
│ Wait for completion signal → show prompt → loop      │
└──────────────────┬─────────────────▲─────────────────┘
                   │ AST (Send)      │ Completion signal
                   ▼                 │
┌─ Main Thread (Tokio LocalSet) ─────┴─────────────────┐
│ Receive AST → create task → run on VM                │
│ Background futures continue running between tasks    │
│ GC, Environment, Values — all stay on this thread    │
│ All output → ExternalPrinter (interactive) or stdout  │
└──────────────────────────────────────────────────────┘
```

**Shell thread:** Runs Reedline (blocking `read_line()`), collects input, parses it via the ish parser, and submits the resulting `Program` AST to the main thread via a channel. The shell thread is responsible only for prompts and command line input — it never displays program output.

**Main thread:** Runs the Tokio runtime with a `LocalSet`. The VM lives here. All `Value` objects, the `Environment`, and GC-managed state are confined to this thread.

**Key property:** Futures spawned via `let x = spawn do_something()` are tasks on the `LocalSet`. When the shell-submitted task completes and returns control to the shell, the spawned futures continue running on the `LocalSet`. The `Future` value is bound to `x` in the VM's environment, which persists across shell submissions.

### Non-Interactive Mode

When ish is started with a file or inline code, there is no shell thread. The main thread parses the input, creates a Tokio runtime and `LocalSet`, runs the program to completion, and exits. The `LocalSet` runs until all tasks complete (or until the top-level program returns, at which point remaining futures follow the `future_drop` rules).

### Communication Between Threads

- **Shell → Main:** A channel carrying `Program` AST nodes. The AST is `Send`-safe (composed of `String`, `i64`, `f64`, `bool`, `Vec<T>`, `Box<T>`, `Option<T>` — no `Gc<>` or `Rc<>` types).
- **Main → Shell:** A channel carrying completion signals. No display content crosses this boundary.
- **Main → Terminal:** All program output goes through the Reedline `ExternalPrinter` in interactive mode, or directly to OS stdout/stderr in non-interactive mode.

```rust
// Sketch of the channel types
enum TaskSubmission {
    Execute(Program),
    Shutdown,
}

enum TaskCompletion {
    Done,       // Task finished; output already routed via ExternalPrinter
    Error,      // Task errored; error already routed via ExternalPrinter
}
```

### Parser Placement

The parser runs on the shell thread because:

1. Reedline's multiline input validator already calls the parser for validation.
2. The parser is stateless — `ish_parser::parse(input: &str) -> Result<Program, Vec<ParseError>>`.
3. The AST (`Program`) is `Send` — it can cross the thread boundary safely.
4. The shell thread is otherwise idle during parsing.
5. In non-interactive mode, there is no shell thread; the main thread parses and executes.

### Output Routing

The shell thread is responsible only for prompts and command line input. All program output — expression results, `println`, error messages, background task output — goes through stdout/stderr, never through a result channel to the shell thread.

In interactive mode, stdout writes use Reedline's `ExternalPrinter` API, which allows the main thread to inject output into the terminal while the Reedline prompt is active. Reedline handles redrawing the prompt after output is displayed.

Code that writes to stdout auto-detects whether the VM is running in interactive mode:
- **Interactive mode:** Write through the `ExternalPrinter`.
- **Non-interactive mode:** Write directly to OS stdout.

This is internal to the runtime — ish user code always calls `println("hello")` regardless of mode.

### Error Display

- Parse errors are detected on the shell thread (before submission) and written to stderr directly.
- Runtime errors are detected on the main thread and formatted to strings on the main thread before output via the `ExternalPrinter` (interactive) or directly to stderr (non-interactive).
- `RuntimeError` values containing `Gc<>` types are formatted to strings on the main thread — GC-managed values never cross thread boundaries.

### Shell Command Execution

Shell command execution migrates from `std::process::Command` to `tokio::process::Command` to avoid blocking the `LocalSet` thread.

At low assurance, shell commands still look synchronous:

```ish
let output = $(ls -la)  // implicitly awaited
```

### Interpreter Architecture Impact

The tree-walking interpreter becomes async-aware. Each evaluation step that might yield (function calls, loop iterations) is an `.await` point. The interpreter's `eval` function becomes `async fn eval(...)`.

---

## Open Questions

- [ ] **Program exit with running futures.** When the user exits the shell (Ctrl+D), or when a file/inline program completes with spawned futures still running, the runtime must decide: (a) wait for all futures, (b) cancel all futures (triggering `future_drop` discrepancies if enabled), (c) wait with a timeout then cancel.
- [ ] **FFI and blocking.** If ish supports calling arbitrary C/Rust functions via FFI, those functions might block. At that point, a `blocking` entry could be reintroduced, or all FFI calls could be required to go through parallel shims.

---

## Referenced by

- [docs/spec/INDEX.md](INDEX.md)
- [docs/spec/assurance-ledger.md](assurance-ledger.md)
- [docs/spec/types.md](types.md)
- [docs/spec/syntax.md](syntax.md)
- [docs/spec/execution.md](execution.md)
- [docs/spec/memory.md](memory.md)
- [docs/spec/modules.md](modules.md)
- [docs/spec/errors.md](errors.md)
- [docs/architecture/overview.md](../../docs/architecture/overview.md)
- [docs/architecture/vm.md](../../docs/architecture/vm.md)
- [docs/architecture/shell.md](../../docs/architecture/shell.md)
- [docs/project/open-questions.md](../project/open-questions.md)
- [docs/user-guide/concurrency.md](../user-guide/concurrency.md)
- [docs/ai-guide/playbook-concurrency.md](../ai-guide/playbook-concurrency.md)
- [docs/project/plans/concurrency.md](../project/plans/concurrency.md)
