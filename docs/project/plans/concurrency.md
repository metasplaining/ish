---
title: "Plan: Concurrency"
category: plan
audience: [ai-dev]
status: completed
last-verified: 2026-03-31
depends-on:
  - docs/project/proposals/concurrency.md
  - docs/spec/assurance-ledger.md
  - docs/spec/execution.md
  - docs/spec/memory.md
  - docs/spec/types.md
  - docs/spec/syntax.md
  - docs/spec/modules.md
  - docs/spec/errors.md
  - docs/architecture/vm.md
  - docs/architecture/shell.md
---

# Plan: Concurrency

*Derived from [concurrency.md](../proposals/concurrency.md) on 2026-03-31.*

## Overview

Implement the concurrency design for ish: cooperative multitasking via async/await on a Tokio runtime with `LocalSet`, guaranteed yield mechanism, parallelism via parallel shims, data transfer and I/O abstractions (design only — implementation of I/O limited to `println`), the two-thread shell/VM architecture, and all associated specification, documentation, and glossary updates.

## Requirements

Extracted from the accepted design proposal (23 decisions). Each requirement is a testable statement.

### Feature 1: Cooperative Multitasking (Decisions 1–4, 6, 10–12, 15–17)

- R1.1: ish user code runs inside a Tokio `LocalSet` on the main thread using `spawn_local`.
- R1.2: At low assurance (`streamlined`), all code is treated as `simple + unyielding` by default; `async`, `await`, `spawn`, and `yield` keywords are available but not required.
- R1.3: When an ish function calls an async standard library function at low assurance, the call is implicitly awaited.
- R1.4: At higher assurance (`cautious`, `rigorous`), `async fn` declaration, `await`, and `spawn` are required explicitly.
- R1.5: `spawn` returns a `Future<T>` immediately without suspending; calling `spawn` does not make the caller yielding.
- R1.6: `await` suspends the caller until the awaited future resolves. `await` makes the caller yielding.
- R1.7: When a `Future` is dropped without being awaited, the underlying task is cancelled via `JoinHandle::abort()`.
- R1.8: `defer` blocks and `with` block cleanup still execute in cancelled tasks.
- R1.9: Awaiting a cancelled future returns a cancellation error, catchable via `try`/`catch`.
- R1.10: If a spawned task throws and the `Future` is dropped without being awaited, the error is logged.
- R1.11: Errors propagate through `await` identically to synchronous error propagation.
- R1.12: `concurrent_map` is a standard library function: `async fn concurrent_map<T,U>(items: [T], fn map_fn(T) -> U) -> U[]`. Results are positionally correlated with inputs. Errors are inline (not separate).
- R1.13: `concurrent_for_each` is a standard library function: `async fn concurrent_for_each<T>(items: [T], fn each_fn(T) -> Void) -> (Void|Error)[]`.
- R1.14: `Complexity` is a ledger entry type with values `simple`/`complex`, applying to functions and blocks.
- R1.15: `Yielding` is a ledger entry type with values `yielding`/`unyielding`, applying to functions and blocks.
- R1.16: A function calling any `complex` function is itself `complex`.
- R1.17: A function containing `await` or `yield` is `yielding`. A function calling a `yielding` function (with `await`) is `yielding`.

#### Assurance Features (Decisions 3, 10, 16)

- R1.18: `async_annotation` feature: `optional` (streamlined) / `required` (cautious, rigorous).
- R1.19: `await_required` feature: `optional` (streamlined) / `required` (cautious, rigorous).
- R1.20: `future_drop` feature: `disabled` (streamlined) / `enabled` (cautious, rigorous). When enabled, dropping a future without awaiting is a discrepancy.

### Feature 2: Guaranteed Yield (Decision 5)

- R2.1: At documented yield points (loop back-edges, function call sites, explicit `yield`), the runtime checks a time-based yield budget (~1ms default).
- R2.2: If the budget is exhausted, the runtime inserts `tokio::task::yield_now().await`.
- R2.3: At higher assurance, `yield every N` (statement-count-based) and `@[yield_budget(Xus)]` (custom time threshold) are available.
- R2.4: `@[unyielding]` suppresses yielding for a block.
- R2.5: `guaranteed_yield` feature: `disabled` (streamlined) / `enabled` (cautious, rigorous). When enabled, `complex + unyielding` is a discrepancy.
- R2.6: `yield_control` feature: `time-based only` (streamlined) / `time-based + statement-count` (cautious, rigorous).

### Feature 3: Parallelism (Decisions 6, 13, 14)

- R3.1: Parallel tasks run on Tokio's thread pool via `tokio::spawn`, require `Send`, and cannot access the GC heap.
- R3.2: Parallel shims have the signature `(symbol, value, parent_shim) -> Future<Value>`, unlike regular shims which return `Value`.
- R3.3: No parallel standard library function accepts ish closures.
- R3.4: `Parallel` is a ledger entry type applying to functions, implying `@[yielding]`.
- R3.5: Immutable `ByteBuffer` backed by `bytes::Bytes` crosses threads with O(1) clone. Mutable `ByteBuffer` backed by `bytes::BytesMut` is frozen or moved for parallel transfer.

### Feature 4: Data Transfer, Streams, and I/O (Decisions 14, 18, 19, 20)

- R4.1: `ByteBuffer` is a single ish type abstracting over `&[u8]`/`Bytes`/`BytesMut`; mutability expressed via `Mutable` ledger entry.
- R4.2: `Reader` wraps `AsyncRead`; `Writer` wraps `AsyncWrite`.
- R4.3: `Stream<T>` is a unified type with specialization: `Stream<ByteBuffer>` uses byte-oriented I/O; `Stream<T>` for other types uses channels.
- R4.4: `StreamWriter<T>` is the write half of a stream.
- R4.5: Stream combinators (`map`, `filter`, `take`, `zip`, `lines`, `decode`) work uniformly on all streams.
- R4.6: File, standard I/O, TCP, and UDP types are specified with convenience methods and Tokio mappings.
- R4.7: **Implementation scope (Decision 20):** Only `println` is implemented. It becomes async internally but its ish-level interface is unchanged. All other I/O types are deferred.

### Feature 5: Blocking (Decision 9)

- R5.1: The `blocking` dimension is eliminated from the ish language.
- R5.2: The `Blocking` entry type is not needed; `Parallel` covers the relevant case.
- R5.3: The existing `blocking` feature in the assurance ledger feature state table must be updated to reflect elimination.

### Feature 6: Shell and Execution Architecture (Decisions 21, 22, 23)

- R6.1: In interactive mode, two threads: shell thread (Reedline, parsing) and main thread (Tokio `LocalSet`, VM).
- R6.2: Shell thread parses input and submits `Program` AST (which is `Send`) to the main thread via a channel.
- R6.3: Main thread sends only a completion signal (not display content) back to the shell thread.
- R6.4: Spawned futures survive after the submitting task completes; they continue running on the `LocalSet`.
- R6.5: In non-interactive mode, there is no shell thread — main thread parses and executes.
- R6.6: All program output (expression results, `println`, errors, background task output) goes through stdout/stderr via Reedline's `ExternalPrinter` in interactive mode; directly to OS stdout/stderr in non-interactive mode.
- R6.7: The interpreter's `eval` function becomes `async fn eval(...)`.
- R6.8: Shell command execution migrates from `std::process::Command` to `tokio::process::Command`.
- R6.9: Parse errors are displayed on the shell thread; runtime errors are formatted to strings on the main thread before output via `ExternalPrinter`.

## Authority Order

1. GLOSSARY.md (new terms)
2. Roadmap (set to "in progress")
3. Maturity matrix (update affected rows)
4. Specification docs
5. Architecture docs
6. User guide / AI guide
7. Agent documentation (AGENTS.md, skills)
8. Acceptance tests
9. Code (implementation)
10. Unit tests
11. Roadmap (set to "completed")
12. Maturity matrix (update affected rows)
13. History
14. Index files

## TODO

### Phase 1: Glossary and Project Tracking

- [x] 1. **Add concurrency terms to GLOSSARY.md** — `GLOSSARY.md`
  - Add terms: async function, await, spawn, yield, future, complexity entry, yielding entry, parallel entry, cooperative multitasking, parallel multitasking, parallel shim, LocalSet, yield budget, yield-eligible point, stream, stream writer, reader, writer, byte buffer, codec, shell thread, main thread, ExternalPrinter, completion signal, concurrent_map, concurrent_for_each

- [x] 2. **Update roadmap** — `docs/project/roadmap.md`
  - Add "Concurrency design" to In Progress section
  - Move completed items if any

- [x] 3. **Update maturity matrix** — `docs/project/maturity.md`
  - Add row for "Concurrency" with Designed=✅, Spec Written=❌, Prototyped=❌, Tested=❌, Stable=❌ (will be updated as work progresses)

### CHECKPOINT 1: Verify glossary terms are consistent with proposal terminology. Verify roadmap and maturity matrix updated.

### Phase 2: Specification Documents

- [x] 4. **Create concurrency specification** — `docs/spec/concurrency.md` (new file)
  - Cooperative multitasking model (async/await/spawn/yield semantics)
  - Runtime architecture (Tokio, LocalSet, spawn_local)
  - Low-assurance vs. high-assurance behavior
  - Cancellation semantics (Future drop, defer/with cleanup)
  - Error propagation through await and spawn
  - Concurrent iteration (concurrent_map, concurrent_for_each)
  - Guaranteed yield mechanism (time-based, statement-count, annotations)
  - Parallelism model (parallel shims, no user-defined parallel functions)
  - Data transfer abstractions (ByteBuffer, Reader, Writer, Stream, StreamWriter)
  - I/O type specifications (file, standard I/O, TCP, UDP) — marked as future work except println
  - Shell/VM two-thread architecture
  - Parser placement (shell thread)
  - Output routing (ExternalPrinter)

- [x] 5. **Update assurance ledger spec** — `docs/spec/assurance-ledger.md`
  - Add `Complexity` entry type (values: simple, complex; applies_to: function, block)
  - Add `Yielding` entry type (values: yielding, unyielding; applies_to: function, block)
  - Add `Parallel` entry type (applies_to: function; implies: @[yielding])
  - Add `async_annotation` feature to feature state table (optional | required, with standard defaults)
  - Add `await_required` feature to feature state table (optional | required, with standard defaults)
  - Add `guaranteed_yield` feature to feature state table (disabled | enabled, with standard defaults)
  - Add `yield_control` feature to feature state table (time-based | time-based + statement-count)
  - Add `future_drop` feature to feature state table (disabled | enabled, with standard defaults)
  - Update `sync_async` row — replace with concurrency-specific features or clarify relationship
  - Update `blocking` row — mark as eliminated per Decision 9; replace with `Parallel` entry type
  - Update built-in standards: add concurrency features to `streamlined`, `cautious`, `rigorous` definitions
  - Add `async` to native syntax equivalence table (maps to `@[async]`)

- [x] 6. **Update types spec** — `docs/spec/types.md`
  - Add `Future<T>` type (generic, represents an eventual result)
  - Add `Stream<T>` type (generic, represents an async sequence of values)
  - Add `StreamWriter<T>` type (generic, write half of a stream)
  - Add `Reader` type (wraps AsyncRead)
  - Add `Writer` type (wraps AsyncWrite)
  - Add `ByteBuffer` type (abstracts over bytes::Bytes/BytesMut; mutability via Mutable entry)

- [x] 7. **Update syntax spec** — `docs/spec/syntax.md`
  - Add `async fn` declaration syntax
  - Add `await expr` syntax
  - Add `spawn expr` syntax
  - Add `yield` statement syntax
  - Add `yield every N` syntax (in for/while loops)
  - Add `@[yield_budget(Xus)]` annotation syntax
  - Add `@[unyielding]` annotation syntax

- [x] 8. **Update execution spec** — `docs/spec/execution.md`
  - Describe Tokio runtime integration in each execution configuration
  - Document the two-thread model for interactive shell (shell thread + VM/LocalSet thread)
  - Document non-interactive mode (single thread with Tokio runtime)
  - Note that thin shell becomes async (shell command execution via tokio::process::Command)

- [x] 9. **Update memory spec** — `docs/spec/memory.md`
  - Note GC remains single-threaded on the LocalSet thread
  - Parallel tasks use Rust-managed memory
  - ByteBuffer mutability and freeze semantics
  - Cross-thread data transfer: immutable ByteBuffer via Bytes (O(1) clone), mutable via freeze or move

- [x] 10. **Update modules spec** — `docs/spec/modules.md`
  - Extend shim system documentation with parallel shim variant
  - Document parallel shim signature: (symbol, value, parent_shim) -> Future<Value>

- [x] 11. **Update errors spec** — `docs/spec/errors.md`
  - Add cancellation error type (returned when awaiting a cancelled future)
  - Add future-drop discrepancy (when future_drop feature is enabled)

- [x] 12. **Update spec index** — `docs/spec/INDEX.md`
  - Add row for concurrency.md

### CHECKPOINT 2: Verify all specification documents are internally consistent. Cross-check entry types, features, and types across assurance-ledger.md, types.md, and concurrency.md. Verify syntax additions are consistent with existing syntax patterns.

### Phase 3: Architecture Documents

- [x] 13. **Update architecture overview** — `docs/architecture/overview.md`
  - Add Tokio runtime to the high-level architecture
  - Update crate dependency graph if needed (e.g., tokio dependency flows)

- [x] 14. **Update VM architecture** — `docs/architecture/vm.md`
  - Document async interpreter (eval becomes async)
  - Document yield budget checking at yield-eligible points
  - Document Future value type (wraps JoinHandle from spawn_local)
  - Document println routing (ExternalPrinter in interactive mode, stdout in non-interactive)

- [x] 15. **Update shell architecture** — `docs/architecture/shell.md`
  - Document two-thread model: shell thread (Reedline + parser) and main thread (Tokio LocalSet + VM)
  - Document channel communication: Program submission (shell → main), completion signal (main → shell)
  - Document ExternalPrinter integration for output routing
  - Document parser placement rationale (shell thread: stateless parser, Send AST)
  - Document non-interactive mode (no shell thread)

### CHECKPOINT 3: Verify architecture documents are consistent with specification. Verify all Tokio integration points are documented.

### Phase 4: User and AI Guides

- [x] 16. **Add concurrency user guide** — `docs/user-guide/concurrency.md` (new file)
  - Low-assurance usage (transparent async — no keywords needed)
  - High-assurance usage (async fn, await, spawn, yield)
  - Concurrent iteration (concurrent_map, concurrent_for_each)
  - Future handling (spawn, await, cancellation)
  - Yield control annotations
  - Examples at each assurance level

- [x] 17. **Update user guide index** — `docs/user-guide/INDEX.md`
  - Add row for concurrency.md

- [x] 18. **Add concurrency AI guide playbook** — `docs/ai-guide/playbook-concurrency.md` (new file)
  - Guidance for AI agents writing concurrent ish code
  - Common patterns and antipatterns
  - Assurance feature interactions

- [x] 19. **Update AI guide index** — `docs/ai-guide/INDEX.md`
  - Add concurrency playbook reference

### CHECKPOINT 4: Verify user guide examples are valid ish syntax per the updated syntax spec. Verify AI guide is consistent with specification.

### Phase 5: Agent Documentation

- [x] 20. **Update AGENTS.md** — `AGENTS.md`
  - Add concurrency-related tasks to the Task Playbooks table (e.g., "Working on concurrency" → relevant spec/arch files)
  - Update "Key Concepts" if needed

### Phase 6: Resolve Open Questions

- [x] 21. **Update open questions** — `docs/project/open-questions.md`
  - Mark "No description of concurrency / parallelism" as resolved
  - Add new open questions from the proposal:
    - Program exit with running futures (wait, cancel, or timeout?)
    - FFI and blocking (if ish supports arbitrary C/Rust calls, blocking may need reintroduction)

### CHECKPOINT 5: Verify all open questions from the proposal are tracked. Run feature coherence audit on the concurrency feature across glossary, specs, architecture, and guides.

### Phase 7: Roadmap Completion and History

- [x] 22. **Update roadmap** — `docs/project/roadmap.md`
  - Move "Concurrency design" to Completed section

- [x] 23. **Update maturity matrix** — `docs/project/maturity.md`
  - Update "Concurrency" row: Designed=✅, Spec Written=✅

- [x] 24. **Update history index** — `docs/project/history/INDEX.md`
  - Verify 2026-03-23-concurrency entry exists (it does per proposal's history checklist)

### Phase 8: Index Files

- [x] 25. **Update docs index** — `docs/INDEX.md`
  - Verify concurrency.md is linked from spec section

- [x] 26. **Update Referenced by sections** — all modified files
  - Add cross-references to/from concurrency.md in all files that now depend on it

### CHECKPOINT 6: Final coherence check. Verify all TODO items complete. Verify all cross-references valid. Verify no broken links.

## Reference

### Decisions Summary (from proposal Decision Register)

| # | Decision | Outcome |
|---|----------|---------|
| 1 | Duration entry taxonomy | Three orthogonal dimensions: Complexity (simple/complex), Yielding (yielding/unyielding), Blocking (eliminated) |
| 2 | Terminology | Standard keywords: `async`, `await`, `spawn`, `yield`, `Future`. Entry types: `Yielding`, `Complexity` |
| 3 | Default behavior | Default to await. Low assurance: async/await/yield not required |
| 4 | Cooperative multitasking model | Standard async/await (not structured concurrency) |
| 5 | Yield mechanism | Hybrid: time-based (low assurance), fine-grained control (high assurance) |
| 6 | Parallelism and runtime | Tokio-based. LocalSet/spawn_local for concurrency. tokio::spawn for parallelism (Rust only). No user-defined parallel functions. Blocking eliminated |
| 7 | Data sharing for parallel code | N/A — superseded by decision 6 |
| 8 | Concurrency and memory management | N/A — superseded by decision 6 |
| 9 | Blocking | Eliminated. No blocking functions in ish. All I/O async via Tokio |
| 10 | Dropped future discrepancy | Yes — `future_drop` assurance feature |
| 11 | Concurrent for | Library function (`concurrent_map`), not syntax |
| 12 | Does spawn make caller yielding? | No — spawn returns Future immediately |
| 13 | Parallel closures | Eliminated — no parallel function accepts ish closures |
| 14 | Parameter marshaling and streams | Zero-copy byte buffers. Stream type required |
| 15 | Entry terminology | `Yielding`/`Complexity` — rename Duration to Complexity |
| 16 | future_drop feature levels | Enabled/disabled only (no separate discrepancy-without-error) |
| 17 | concurrent_map signature | Inline results, positionally correlated |
| 18 | Stream model | Unified with specialization (Stream<ByteBuffer> vs Stream<T>) |
| 19 | I/O interface model | Core abstractions: Reader, Writer, Stream, StreamWriter, ByteBuffer. Convenience methods on I/O types |
| 20 | I/O scope | println only — all other I/O deferred |
| 21 | Shell/VM threading | Two threads in interactive mode. Shell thread + main/VM thread |
| 22 | Parser placement | Parser runs on shell thread. AST is Send |
| 23 | Output routing | Reedline ExternalPrinter for interactive mode. No result channel to shell thread |

### Existing Features That Need Updates

The assurance ledger feature state table currently has `sync_async` and `blocking` rows. Per the proposal:
- `sync_async` should be replaced or updated to reference the new `async_annotation` and `await_required` features.
- `blocking` should be marked as eliminated and replaced with the `Parallel` entry type.

### Built-In Standard Updates

Current built-in standards need these concurrency features added:

```ish
standard streamlined [
    // existing features...
    async_annotation(optional),
    await_required(optional),
    guaranteed_yield(disabled),
    future_drop(disabled),
]

standard cautious [
    // existing features...
    async_annotation(required, runtime),
    await_required(required, runtime),
    guaranteed_yield(enabled),
    future_drop(enabled),
]

standard rigorous extends cautious [
    // existing features...
    async_annotation(required, build),
    await_required(required, build),
    guaranteed_yield(enabled),
    future_drop(enabled),
    yield_control(time_and_count),
]
```

### Scope Boundaries

This plan covers documentation and specification only. No prototype code changes are included — the proposal's I/O implementation scope (Decision 20) limits implementation to `println`, but that is a code task for a future plan. This plan establishes the authoritative specification that a code implementation plan would follow.

### Files Modified or Created

| File | Action |
|------|--------|
| `GLOSSARY.md` | Modified — add ~25 terms |
| `AGENTS.md` | Modified — add concurrency task playbook |
| `docs/project/roadmap.md` | Modified — add concurrency milestone |
| `docs/project/maturity.md` | Modified — add concurrency row |
| `docs/project/open-questions.md` | Modified — resolve one, add two |
| `docs/spec/concurrency.md` | **Created** — full concurrency specification |
| `docs/spec/assurance-ledger.md` | Modified — add entry types, features, standard updates |
| `docs/spec/types.md` | Modified — add Future, Stream, StreamWriter, Reader, Writer, ByteBuffer |
| `docs/spec/syntax.md` | Modified — add async/await/spawn/yield syntax |
| `docs/spec/execution.md` | Modified — add Tokio runtime, two-thread model |
| `docs/spec/memory.md` | Modified — add GC threading, ByteBuffer, cross-thread transfer |
| `docs/spec/modules.md` | Modified — add parallel shim variant |
| `docs/spec/errors.md` | Modified — add cancellation error, future-drop discrepancy |
| `docs/spec/INDEX.md` | Modified — add concurrency row |
| `docs/architecture/overview.md` | Modified — add Tokio runtime |
| `docs/architecture/vm.md` | Modified — async interpreter, yield budget, Future value, println routing |
| `docs/architecture/shell.md` | Modified — two-thread model, channels, ExternalPrinter |
| `docs/user-guide/concurrency.md` | **Created** — concurrency user guide |
| `docs/user-guide/INDEX.md` | Modified — add concurrency row |
| `docs/ai-guide/playbook-concurrency.md` | **Created** — AI concurrency playbook |
| `docs/ai-guide/INDEX.md` | Modified — add playbook reference |
| `docs/project/history/INDEX.md` | Verified — entry already exists |
| `docs/INDEX.md` | Modified — verify concurrency linked |

---

## Referenced by

- [docs/project/plans/INDEX.md](INDEX.md)
