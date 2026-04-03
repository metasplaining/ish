---
title: "History: Runtime Extraction"
category: history
audience: [all]
status: final
last-verified: 2026-04-02
depends-on: [docs/project/proposals/runtime-extraction.md, docs/project/plans/runtime-extraction.md]
---

# Runtime Extraction — Design History

## Summary

The runtime extraction proposal began with a straightforward request: move the `Shim` and `Value` types from `ish-vm` to `ish-runtime` so that compiled ish packages can depend on the runtime crate without pulling in the full interpreter.

The initial proposal (v1) analyzed the transitive closure of `Value` and found that `Environment`, `IshFunction`, `FunctionImplementation`, `FutureRef`, and `RuntimeError` all needed to move together. It identified three alternatives: move everything (Alternative A), create an intermediate crate (Alternative B), or partial extraction (Alternative C). The analysis recommended Alternative A but flagged significant concerns: `ish-runtime` would gain dependencies on `gc`, `tokio`, `crossbeam`, and `ish-ast`, and `Environment` — a type deeply tied to the interpreter's variable lookup — would need to move to the runtime.

The review revealed a deeper insight. The difficulties all originated from `Value::Function`, which contained `IshFunction`, which contained both `Environment` (interpreter state) and `Statement` (AST node). The reviewer asked the right question: how does a compiled function actually call an interpreted function it receives as an argument? The answer reshaped the proposal: redefine all functions to be shim-based. The `FunctionImplementation::Interpreted` variant would be eliminated. When the VM encounters a function declaration, it wraps the body in a shim closure that captures the environment, body, and VM reference — everything needed for interpreted execution. Compiled code simply calls the shim without knowing or caring how the function was originally defined.

This architectural change cleanly breaks the dependency chain. `IshFunction` no longer needs `Environment` or `Statement` as fields — both are captured inside the shim closure. `ish-runtime` no longer needs `ish-ast` or `crossbeam` as dependencies. The extraction becomes straightforward.

The review also decided to add an `ErrorCode` enum immediately rather than deferring it, and to remove the unused `IshValue` type. A new requirement was added: testing the shim-only architecture with deep interpreted→compiled→interpreted nesting to catch recursion bugs that might only manifest at depth.

The v2 proposal reorganizes around five features with clear dependencies: remove `IshValue` first, then implement the shim-only architecture, validate with nesting tests, add the error code enum, and finally perform the actual extraction.

## Versions

- [v1.md](v1.md) — Initial proposal with three alternatives for extraction strategy.
- [v2.md](v2.md) — Shim-only architecture, five-feature structure, VM self-reference options analyzed.

## v2 → v3 Changes

The v2 review settled three open questions.

First, the VM self-reference problem. The v2 proposal presented five options for how interpreted-function shims would call back into the VM and deferred the choice to the implementation plan. The reviewer decided firmly: adopt `Rc<RefCell<IshVm>>` globally, replacing the current `&mut self` pattern throughout the interpreter. The discipline is simple — borrow mutably only when mutation is needed, release the borrow before calling any shim. This makes double-borrow prevention easy to verify by inspection. The decision was promoted from "deferred" to a firm architectural commitment in the v3 decision register (Decision 11).

Second, the `TypeAnnotation` dependency. The v2 proposal noted that `IshFunction` references `ish_ast::TypeAnnotation` for parameter and return types, and recommended allowing `ish-runtime` to depend on `ish-ast` since it's a lightweight crate. The reviewer chose a different path: extract `TypeAnnotation` from `ish-ast` into a new `ish-core` crate. Both `ish-ast` and `ish-runtime` depend on `ish-core`, keeping the dependency graph clean. This adds a new crate but avoids coupling the runtime to the parser's AST representation. Decision 8 was updated to reflect `ish-core` instead of `ish-ast`, and new Decision 12 was added.

Third, cross-boundary error propagation testing. The v2 proposal included a test case for errors thrown by interpreted functions propagating back through compiled shims. The reviewer noted that error propagation across the compiled layer is a behavior of the compiled function, not the interpreter, and directed that this test be skipped for now. Decision 10 was updated accordingly.

The v3 proposal integrates these three decisions, restructures Feature 2 to include the `ish-core` crate creation as Phase A, and updates the dependency diagram and documentation lists to reflect the new crate.

## Acceptance

The v3 proposal was accepted with all 12 decisions settled. The final design comprises five features executed in dependency order: (1) remove IshValue, (2) shim-only function architecture with `Rc<RefCell<IshVm>>`, (3) cross-boundary nesting tests, (4) error code enum, and (5) create ish-core crate and move Value/Shim/RuntimeError/IshFunction to ish-runtime.

## Versions

- [v1.md](v1.md) — Initial proposal with three alternatives for extraction strategy.
- [v2.md](v2.md) — Shim-only architecture, five-feature structure, VM self-reference options analyzed.
- [v3.md](v3.md) — All decisions settled: `Rc<RefCell<IshVm>>`, `ish-core` crate, error propagation test deferred. Accepted.

## Implementation

*Completed 2026-04-02.*

The implementation plan had 65 TODOs across 11 phases, executed in authority order (glossary → docs → tests → code → finalize).

**Phases 0–4** front-loaded documentation and test changes. The glossary gained six new/updated terms, spec and architecture docs were rewritten to describe the target state, and cross-boundary acceptance tests were written (expected to fail until Phase 7). This ensured the documentation always described the intended final shape.

**Phase 5** deleted the vestigial `IshValue` enum from `ish-runtime` — trivial housekeeping.

**Phase 6** was the largest and riskiest phase. Sub-phase 6A converted every `IshVm` method from `&mut self` to associated functions taking `vm: &Rc<RefCell<IshVm>>`. Roughly 20 method signatures changed simultaneously across the interpreter, stdlib, and shell. The key discipline: keep borrows brief, never hold `borrow_mut()` across shim calls. For ledger operations that both read and mutate, block scoping prevented double-borrow panics.

Sub-phase 6B eliminated the `FunctionImplementation` enum. All functions became shim-based. For interpreted functions, the shim captures body and environment, stores them in a thread-local `PENDING_INTERP_CALL`, and returns immediately. The caller (`call_function_inner`) checks the thread-local and executes the body asynchronously with proper defer/yield/audit handling. This thread-local mechanism bridges the sync `Shim` interface with the async interpreter.

**Phase 7** added the `apply(fn, args_list)` builtin for cross-boundary testing. Like ledger builtins, it's intercepted by name in `call_function_inner`. A `has_yielding_entry.is_some()` guard prevents hijacking user-defined functions named `apply`. The cross-boundary tests — 5 tests covering up to 6+ boundary crossings and closure capture — all passed.

**Phase 8** introduced the `ErrorCode` enum with 13 variants (E001–E013), replacing 188 string literal error codes. `RuntimeError::system_error()` now accepts `ErrorCode` instead of `&str`.

**Phase 9** created `ish-core` (shared `TypeAnnotation`) and moved `value.rs`/`error.rs` to `ish-runtime`. The `ish-vm` crate re-exports `ish_runtime::value` and `ish_runtime::error` for backward compatibility, so no downstream imports changed.

The prototype now has 8 crates. Compiled ish packages depend only on `ish-runtime` (which depends on `ish-core`, `gc`, and `tokio`) — no interpreter, parser, or AST dependency. All 345 unit tests and 260 acceptance tests pass.
