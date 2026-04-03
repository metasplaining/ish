---
title: "History: Concurrency Design"
category: history
audience: [all]
status: draft
last-verified: 2026-03-29
depends-on: [docs/project/proposals/concurrency.md, docs/project/rfp/concurrency.md]
---

# Concurrency Design

## Summary

The concurrency proposal began with a prompt file describing two mechanisms: cooperative multitasking (async with assurance ledger integration) and parallel multitasking (OS threads). The prompt raised several open questions about entry taxonomy, terminology, and design approach.

### v1 — Initial Proposal (2026-03-23)

The initial proposal analyzed the prompt and identified that the five RFP entries (simple, complex, yielding, unyielding, blocking) were not all in the same dimension, suggesting three orthogonal axes. It presented four alternatives for cooperative multitasking (standard async/await, goroutine-style, structured concurrency, and a hybrid), four alternatives for the guaranteed yield mechanism, five alternatives for parallelism (thread spawn, shared-state, parallel iterators, structured parallelism, actor model), and two alternatives for blocking detection. The proposal recommended a hybrid approach combining standard async/await with structured concurrency primitives, structured parallelism with scoped threads, and hybrid time-based yielding. Nine decisions were left pending for human review.

### v2 — Revised Proposal (2026-03-28)

The human reviewed v1 and made all nine decisions, fundamentally reshaping the design:

**Cooperative multitasking** was simplified to standard async/await (Alternative A). The structured concurrency `concurrent` block was rejected because variables declared inside the block would be out of scope after it exits, creating surprising semantics for returning values. The human noted that a `concurrent for` loop might be useful but questioned whether it offered practical advantages over a library function.

**The Tokio runtime model** was the biggest change. Rather than designing a custom concurrency runtime, the human decided to build ish's concurrency on Tokio. Cooperative tasks run on a `LocalSet` with `spawn_local`, which avoids `Send` requirements and lets GC-managed objects be shared freely. Parallel tasks run on Tokio's thread pool via `spawn`, but there is no way to define parallel functions in ish — they exist only in the standard library, implemented in Rust with parallel shims. This decision elegantly sidesteps the hardest problems in concurrent language design: data races, GC thread safety, and `Send`/`Sync` requirements become the Rust library implementer's responsibility, not the ish developer's.

**Blocking was eliminated entirely.** Since all ish I/O goes through Tokio async functions, and any synchronous Rust library wraps its I/O in parallel shims, there are no blocking functions in the ish language.

**Low-assurance behavior** was significantly softened. At low assurance, all code looks synchronous — the runtime automatically awaits async functions and inserts yield points. The `async`, `await`, and `yield` keywords exist but are not required until higher assurance levels.

The revised proposal raises three new concerns: (1) how parallel collection functions interact with ish closures (since ish closures are `!Send` and can't run on the thread pool), (2) parameter marshaling overhead for large data, and (3) the significant refactoring required to make the tree-walking interpreter async-aware.

### v3 — Second Revision (2026-03-28)

The human reviewed v2 and made six additional decisions, further sharpening the design.

**Dropped future discrepancy** (Decision 10): The human confirmed that dropping a `Future` without awaiting it should be a discrepancy at higher assurance levels. A new `future_drop` assurance feature was added with three states: disabled (streamlined), discrepancy (cautious), and error (rigorous).

**Concurrent for** (Decision 11): Decided as a library function rather than syntax. The key insight was that concurrent iteration needs to run all tasks to completion and return a list of errors — behavior that would be surprising for a `for` loop but natural for a function call like `concurrent_for_each`. This resolved one of the open questions from v2.

**Spawn does not yield** (Decision 12): A subtle but important correction to the inference rules. Calling `spawn` returns a `Future` immediately without suspending, so it does not make the calling function `yielding`. Only `await` and `yield` do.

**No parallel closures** (Decision 13): The most consequential decision in this round. The human struck all parallel standard library functions that accept closures (`parallel_map`, `parallel_filter`, `parallel_for_each`, `parallel_reduce`). The reasoning was clean: accepting ish closures in parallel functions pulls all the parallelism complexity (thread safety, `Send`/`Sync`, data race prevention) right back into the language, defeating the purpose of the Tokio-based model. CPU-bound work that needs closures uses `concurrent_for_each` on the `LocalSet` (single-threaded but interleaved). True multi-threaded parallelism is reserved for Rust-implemented I/O and computation that doesn't need ish closures.

**Marshaling and streams** (Decision 14): The human laid out a detailed vision for data transfer between threads. The `Value` enum should discriminate between mutable and immutable byte buffers (using `bytes::Bytes` and `bytes::BytesMut` from the Tokio ecosystem) to enable zero-copy sharing. For complex types, copy-in/copy-out semantics in the shim. Most importantly, the human identified streams as the primary data transfer mechanism between the main thread and I/O threads, and requested a thorough analysis of stream alternatives. Three alternatives were proposed: (A) channel-based streams using `tokio::sync::mpsc`, (B) separate `ByteStream`/`ObjectStream` types using Tokio's `AsyncRead`/`AsyncWrite` traits, and (C) a unified `Stream<T>` trait with specialization. Alternative A was recommended for its simplicity: both halves are `Send`, backpressure is natural, and the model covers both byte and typed data. A stream decision remains pending.

**Entry terminology** (Decision 15): The `Duration` entry type was renamed to `Complexity`, keeping `Yielding` as-is. The names now describe the runtime properties: `Complexity` captures whether code is `simple` or `complex`, and `Yielding` captures whether it suspends.

### v4 — Third Revision (2026-03-29)

The human reviewed v3 and made three additional decisions, resolving the stream question and tightening two areas of the cooperative multitasking design.

**`future_drop` simplification** (Decision 16): The human pointed out that discrepancies already produce runtime errors in the assurance ledger — there is no mode where a discrepancy is merely "reported" without becoming an error. The three-level `future_drop` table (disabled / discrepancy / discrepancy+error) in v3 was therefore a false distinction. The feature was simplified to a binary toggle: disabled at `streamlined`, enabled at `cautious` and `rigorous`.

**`concurrent_map` signature and error handling** (Decision 17): The v3 proposal showed `concurrent_map` returning a `(results, errors)` tuple, which the human rejected. The correct signature is `async fn concurrent_map<T,U>(items: [T], fn map_fn(T) -> U) -> U[]`. Results are positionally correlated with inputs — `results[i]` corresponds to `items[i]`. When the mapping function can throw, `U` is a union type like `ValidResult|Error`, and the returned array contains either a result or an error at each position. There is no separate errors collection. `concurrent_for_each` follows the same pattern: `async fn concurrent_for_each<T>(items: [T], fn each_fn(T) -> Void) -> (Void|Error)[]`. This design preserves the crucial input-output correspondence that would be lost with a separate errors list.

**Unified stream model with specialization** (Decision 18): The human chose Alternative C (unified stream with specialization) over the recommended Alternative A (channel-based). The reasoning was layered: low-assurance ish should be easy to use with a single `Stream<T>` abstraction, while high-assurance ish should access the fastest I/O Tokio provides — zero-copy `Bytes` for network I/O via the `bytes` crate. The transition from low to high assurance should require adding ledger entries, not rewriting I/O code. This means the abstraction must hide significant implementation differences: network I/O natively supports `BytesMut`/`Bytes` for zero-copy; file I/O uses `spawn_blocking` with `Vec<u8>` internally; stdin/stdout also use `spawn_blocking`. Memory-mapped files might enhance file I/O performance but are not natively async.

The human specified key architectural constraints: ish's standard library I/O functions should wrap Tokio ecosystem types directly (like `Stdin`, `TcpStream`, `tokio::fs::File`), not spawn their own tasks. The standard library should lay cleanly on top of Tokio's normal I/O mechanisms.

In response, the proposal was expanded with a detailed Tokio I/O mapping table showing how each I/O source (stdin, stdout, file, TCP, UDP) maps to Tokio types, their traits, buffer types, and zero-copy capabilities. Three alternative I/O interface designs were proposed:

- **Alternative I (Stream-First):** Everything is `Stream<Bytes>` — simplest mental model but no direct byte-level reads, no seeking, and overhead for simple read-all patterns.
- **Alternative II (Reader/Writer Foundation with Stream Adapter):** `Reader`/`Writer` types wrapping `AsyncRead`/`AsyncWrite`, with `.stream()` adapters — provides direct byte access and seeking, but two concepts to learn.
- **Alternative III (Convenience Functions with Lazy Streaming):** Top-level functions like `file.read()` and `file.write()` for simple patterns, with handles that convert to streams for incremental processing — matches the assurance continuum (simple at low, precise at high).

Alternative III was recommended as best fitting ish's assurance model: `file.read()` maps to `tokio::fs::read()` (one call, no streams), while `handle.stream()` provides full streaming with zero-copy for network I/O. The I/O interface decision remains pending.

### v5 — Fourth Revision (2026-03-29)

The human reviewed v4 and made one decision resolving the I/O interface model, the last pending question from that version.

**I/O interface model** (Decision 19): The human specified five core abstractions — `Reader`, `Writer`, `Stream`, `StreamWriter`, and `ByteBuffer` — and declared that streams must be composable. Rather than choosing one of the three proposed alternatives (stream-first, reader/writer + adapter, or convenience + lazy streaming), the human selected a hybrid: I/O types (stdin, stdout, File, TcpSocket, UdpSocket) should have convenience methods for both whole-resource operations (like reading an entire file) and for opening streams directly, without requiring the intermediate step of creating a Reader and then converting it. This means `file.read()` returns a `ByteBuffer` for simple cases, while `file.stream()` returns a `Stream<ByteBuffer>` directly — no manual `Reader → .stream()` composition needed for the common streaming case.

The human also directed that `&[u8]` and `Bytes` — which have different semantics in Rust — should be abstracted into a single `ByteBuffer` type in ish "if possible." The proposal resolved this by leveraging ish's existing `Mutable` ledger entry: an immutable `ByteBuffer` is backed by `bytes::Bytes` (reference-counted, O(1) clone, zero-copy), while a mutable `ByteBuffer` (carrying `@[Mutable]`) is backed by `bytes::BytesMut` (unique ownership, efficient mutation). The `freeze()` method converts mutable to immutable in O(1). This means the Rust distinction is real at the implementation level but invisible to the ish developer at low assurance — the developer just works with `ByteBuffer`, and the ledger tracks mutability. At higher assurance, the `Mutable` entry makes the semantics explicit without introducing a second type.

Finally, the human requested a minimal I/O library subset to be defined for implementation alongside concurrency. The proposal defined a concrete table of 30+ functions/methods covering `ByteBuffer`, `Reader`, `Writer`, `Stream<T>`, `StreamWriter<T>`, file I/O, standard I/O, and basic TCP — the minimum needed to validate the complete design from convenience functions through streaming with combinators.

With Decision 19, all I/O interface questions are resolved. The concurrency proposal has no remaining pending decisions.

### v6 — Fifth Revision (2026-03-29)

The human reviewed v5 and scoped the I/O implementation: the full I/O library design (ByteBuffer, Reader, Writer, Stream, StreamWriter, file I/O, stdin/stdout helpers, TCP) should be preserved in the proposal as the target design, but all of it is out of scope for the concurrent implementation. Only `println` needs to be implemented with concurrency, and its interface should remain unchanged from the current prototype.

**I/O implementation scope** (Decision 20): This decision draws a clear line between design and implementation. The I/O types and their Tokio mappings stay in the proposal as a reference for future work, but the concurrent implementation focuses exclusively on making the existing `println` work in the async runtime. No new I/O types, no ByteBuffer, no streaming — just `println`.

### Accepted (2026-03-29)

With all 20 decisions resolved and no open `-->` markers remaining, the proposal was accepted. The body was already in settled-fact form after six rounds of revision — no alternatives or decision prompts needed to be removed. The frontmatter status was set to "accepted" and the proposal moved to implementation planning.

The final design covers five features: cooperative multitasking via async/await on a Tokio `LocalSet`, a hybrid guaranteed yield mechanism, parallelism via Rust-only parallel shims on the Tokio thread pool, a comprehensive data transfer and I/O model (with ByteBuffer, Reader, Writer, Stream, StreamWriter), and the elimination of blocking from the language. Of these, only the concurrency runtime and `println` are in scope for the initial implementation (Decision 20); the full I/O library is preserved as future work.

### Implementation Plan and v7 (2026-03-29)

A comprehensive implementation plan was generated from the accepted proposal, producing 34 requirements traced to the decision register and 44 TODO items organized into nine phases with six checkpoints. The plan covered authority-ordered changes from GLOSSARY.md through unit tests, targeting all five features plus the `println`-only I/O scope.

The planning process immediately exposed two architectural gaps in the proposal. First, Decision 20 stated that `println`'s "interface remains unchanged," but this was imprecise — `println` must become an async function internally to work correctly inside the Tokio `LocalSet`. The ish-level interface (calling convention, behavior) is unchanged, but the implementation must be async-aware. Second, and more fundamentally, the proposal assumed the VM would borrow the main thread from the shell during execution, which means the Tokio `LocalSet` would be blocked during shell input. This prevents background tasks from continuing while the user types — a core requirement for any useful concurrent runtime.

### v8 — Sixth Revision (2026-03-29)

The human identified the two problems exposed by the implementation plan and directed a revision.

**`println` becomes async** (Decision 20, clarified): The decision was updated to state that `println` "becomes async internally but its ish-level interface and behavior are unchanged." This is a clarification, not a change in intent — the original decision always meant that ish developers should not see a difference, but the implementation must be async to participate in the Tokio runtime.

**Two-thread model** (Decision 21): The shell and VM must run on separate threads. The shell thread runs Reedline (whose `read_line()` is blocking and cannot be made async) and the parser (which is stateless and produces `Send`-safe AST). The main thread runs the Tokio `LocalSet` with the VM, GC-managed values, and all async tasks. Communication flows in two directions: `Program` ASTs from the shell thread to the main thread (Send-safe, containing only String, i64, f64, bool, Vec, Box, Option), and result information from the main thread back to the shell thread.

**Parser on the shell thread** (Decision 22): The parser was placed on the shell thread rather than the main thread. The rationale: the parser is already needed on the shell thread for Reedline's multiline validation (the `IshValidator` calls `ish_parser::parse()` to determine if input is complete), the parser is stateless with no dependency on VM state, and the AST is `Send`-safe so it crosses the thread boundary cleanly. Parsing on the main thread would add an unnecessary round trip.

A new Feature 6 (Shell and Execution Architecture) was added to the proposal, describing the two-thread model in detail: thread responsibilities, communication channels, lifecycle, and non-interactive mode (which remains single-threaded since there is no REPL to block). The feature includes a concerns analysis covering GC confinement (all `Gc<GcCell<>>` types stay on the main thread), stdout coordination (three alternatives proposed), error display routing, and shutdown sequencing.

One new pending decision was introduced: which stdout coordination approach to use when both the REPL and async tasks need to write to the terminal. Three alternatives were proposed — (A) Reedline's `ExternalPrinter` API, which injects output into the terminal without corrupting the prompt; (B) a dedicated output channel with the shell thread owning all terminal writes; and (C) mutex-guarded stdout. Alternative A was recommended as the cleanest integration.

The proposal status was reverted from "accepted" to "proposal" pending the stdout coordination decision. The implementation plan generated from the pre-v8 proposal was deleted as stale — it does not account for the two-thread model.

### v9 — Seventh Revision (2026-03-29)

The human reviewed v8 and made two inline corrections that combined into a single decision.

The first correction addressed the shell thread description: v8 stated the shell "displays the result and loops back to collect more input," but this was wrong. The shell thread is responsible only for prompts and command line input — it never displays program output. All program output (expression results, `println`, error messages, background task output) goes through stdout/stderr, not through a result channel to the shell thread. This insight fundamentally changed the Main→Shell communication channel from carrying display-formatted strings to carrying only completion signals.

The second correction resolved the pending stdout coordination question from v8. The human chose Reedline's `ExternalPrinter` (Alternative A) but added a critical requirement: code that writes to stdout must auto-detect whether the VM is running in interactive (shell) mode and choose between the `ExternalPrinter` and the OS stdout stream. This means the same `println` implementation works in both interactive and non-interactive mode without the caller needing to know the difference.

**Output routing and stdout coordination** (Decision 23): These two corrections were recorded as a single decision because they are inseparable — the output routing model (shell thread never displays output) and the stdout coordination mechanism (ExternalPrinter with mode auto-detection) are two facets of the same architectural choice. The `TaskResult` enum from v8 (carrying `Value(String)`, `Error(String)`, `Null`) was replaced with a simple `TaskCompletion` enum (`Done`, `Error`) since all output has already been routed through the `ExternalPrinter` by the time the completion signal is sent.

With Decision 23, all pending decisions are resolved.

---

## Version History

| Version | Date | Description |
|---------|------|-------------|
| [v1](v1.md) | 2026-03-23 | Initial proposal with 9 pending decisions |
| [v2](v2.md) | 2026-03-28 | All 9 decisions resolved; Tokio-based model adopted; blocking eliminated |
| [v3](v3.md) | 2026-03-28 | 6 additional decisions (10–15); parallel closures eliminated; streams proposed; entry terminology finalized |
| [v4](v4.md) | 2026-03-29 | 3 additional decisions (16–18); `future_drop` simplified; `concurrent_map` signature fixed; unified stream model chosen; I/O interface alternatives proposed |
| [v5](v5.md) | 2026-03-29 | 1 additional decision (19); I/O interface model resolved; `ByteBuffer` unified type; minimal I/O library defined; no remaining pending decisions |
| [v6](v6.md) | 2026-03-29 | 1 additional decision (20); I/O implementation scoped to `println` only; full I/O library preserved as future work |
| [v7](v7.md) | 2026-03-29 | Accepted; implementation plan generated; two architectural gaps identified |
| [v8](v8.md) | 2026-03-29 | 3 decisions (20 clarified, 21–22 new); two-thread shell/VM model; Feature 6 added; 1 pending decision (stdout coordination) |
| v9 | 2026-03-29 | 1 decision (23); output routing via ExternalPrinter; shell thread never displays output; all pending decisions resolved |

---

## Referenced by

- [docs/project/history/INDEX.md](../INDEX.md)
