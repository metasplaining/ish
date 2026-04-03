---
title: "Plan: Concurrency Code and Tests"
category: plan
audience: [ai-dev]
status: completed
last-verified: 2026-03-31
depends-on:
  - docs/project/proposals/concurrency.md
  - docs/project/plans/concurrency.md
  - docs/spec/concurrency.md
  - docs/spec/syntax.md
  - docs/spec/assurance-ledger.md
  - docs/architecture/vm.md
  - docs/architecture/shell.md
---

# Plan: Concurrency Code and Tests

*Derived from [concurrency.md](../proposals/concurrency.md) on 2026-03-31. Continues from the documentation plan [concurrency.md](concurrency.md) (completed).*

## Overview

Implement the concurrency runtime in the ish prototype: new AST nodes for async/await/spawn/yield, parser rules, async interpreter on Tokio with `LocalSet`, `Future` value type, yield budget mechanism, two-thread interactive shell, `println` output routing via `ExternalPrinter`, and acceptance tests. I/O types beyond `println` are out of scope (Decision 20).

## Scope

This plan covers code and tests only. All documentation, glossary, specification, architecture, and guide updates were completed in the prior plan ([concurrency.md](concurrency.md)). Authority order steps 1–7 (glossary through agent docs) are already done. This plan covers steps 8–10 (acceptance tests, code, unit tests) plus final roadmap/maturity updates.

## Requirements

Extracted from the accepted design proposal. Only requirements that involve code or test changes are listed.

### Cooperative Multitasking

- R1.1: ish user code runs inside a Tokio `LocalSet` on the main thread using `spawn_local`.
- R1.2: At low assurance, `async`/`await`/`spawn`/`yield` keywords are available but not required. Async stdlib calls are implicitly awaited.
- R1.4: At higher assurance, `async fn`, `await`, and `spawn` are required explicitly.
- R1.5: `spawn` returns a `Future<T>` immediately without suspending.
- R1.6: `await` suspends the caller until the awaited future resolves.
- R1.7: When a `Future` is dropped without being awaited, the underlying task is cancelled via `JoinHandle::abort()`.
- R1.8: `defer` blocks and `with` block cleanup still execute in cancelled tasks.
- R1.9: Awaiting a cancelled future returns a cancellation error, catchable via `try`/`catch`.
- R1.11: Errors propagate through `await` identically to synchronous error propagation.
- R1.14: `Complexity` is a ledger entry type with values `simple`/`complex`.
- R1.15: `Yielding` is a ledger entry type with values `yielding`/`unyielding`.

### Guaranteed Yield

- R2.1: At yield-eligible points (loop back-edges, function call sites, explicit `yield`), check a time-based yield budget (~1ms default).
- R2.2: If exhausted, insert `tokio::task::yield_now().await`.
- R2.3: `yield every N` and `@[yield_budget(Xus)]` available at higher assurance.
- R2.4: `@[unyielding]` suppresses yielding for a block.

### Shell Architecture

- R6.1: Two threads in interactive mode: shell thread (Reedline, parsing) and main thread (Tokio `LocalSet`, VM).
- R6.2: Shell thread parses input and submits `Program` AST via channel.
- R6.3: Main thread sends completion signal back.
- R6.4: Spawned futures survive after the submitting task completes.
- R6.5: Non-interactive mode: no shell thread, main thread parses and executes.
- R6.6: All output goes through `ExternalPrinter` (interactive) or stdout (non-interactive).
- R6.7: The interpreter's `eval` function becomes `async fn eval(...)`.
- R6.8: Shell commands migrate from `std::process::Command` to `tokio::process::Command`.
- R6.9: Parse errors on shell thread; runtime errors formatted on main thread.

### Assurance Features (ledger integration)

- R1.18: `async_annotation` feature: `optional` / `required`.
- R1.19: `await_required` feature: `optional` / `required`.
- R1.20: `future_drop` feature: `disabled` / `enabled`.
- R2.5: `guaranteed_yield` feature: `disabled` / `enabled`.
- R2.6: `yield_control` feature: `time-based only` / `time-based + statement-count`.

### Out of Scope

- I/O types: ByteBuffer, Reader, Writer, Stream, StreamWriter, file.*, tcp.*, udp.* (Decision 20)
- Parallel shims (no Rust parallel library functions yet)
- `concurrent_map` / `concurrent_for_each` (require async stdlib support)
- Codegen/compilation (ish-codegen, ish-runtime) — async changes deferred

## Authority Order (Remaining Steps)

Steps 1–7 are complete. This plan covers:

8. Acceptance tests
9. Code (implementation)
10. Unit tests
11. Roadmap (status → "completed")
12. Maturity matrix (update affected rows)
13. Index files

Note: Code and tests are naturally interleaved — each phase includes both implementation and tests.

## Current Codebase Summary

Key facts for the implementing agent (avoids re-reading historical files):

### AST (`ish-ast/src/lib.rs`)
- `Program { statements: Vec<Statement> }` — derives `Debug, Clone, PartialEq, Serialize, Deserialize`
- `Statement` enum: 29 variants. Relevant: `FunctionDecl` (has `name`, `params`, `return_type`, `body`, `visibility`, `type_params`), `ExpressionStmt`, `Annotated`, `ShellCommand`, `Defer`, `TryCatch`, `WithBlock`, `ForEach`, `While`
- `Expression` enum: 15+ variants. `FunctionCall`, `Lambda`, `Incomplete`
- `FunctionDecl` has no `async` flag currently
- AST types do NOT derive `Send`/`Sync` but contain only `String`, `Vec<T>`, `Box<T>`, `Option<T>` — they are `Send` by structure

### Parser (`ish-parser/src/lib.rs`)
- `pub fn parse(input: &str) -> Result<Program, Vec<ParseError>>` — stateless PEG parser using pest
- Grammar file: `proto/ish-parser/src/ish.pest`
- Parser builder: `proto/ish-parser/src/builder.rs`

### VM (`ish-vm/`)
- `IshVm { global_env: Environment, defer_stack: Vec<Vec<(Statement, Environment)>>, ledger: LedgerState }`
- `pub fn run(&mut self, program: &Program) -> Result<Value, RuntimeError>` — synchronous
- `fn exec_statement(&mut self, stmt: &Statement, env: &Environment) -> Result<ControlFlow, RuntimeError>`
- `pub fn eval_expression(&mut self, expr: &Expression, env: &Environment) -> Result<Value, RuntimeError>`
- `pub fn call_function(&mut self, func: &Value, args: &[Value]) -> Result<Value, RuntimeError>`
- All methods take `&mut self` — single owner
- `Value` enum: Bool, Int, Float, String(Rc), Char, Null, Object(Gc), List(Gc), Function(Gc), BuiltinFunction(Rc)
- `Environment` uses `Gc<GcCell<Scope>>` — NOT `Send`
- `IshFunction` stores AST `Statement` for body (which IS `Send`) and `Environment` for closure (NOT `Send`)
- The `gc` crate (v0.5) is fundamentally single-threaded (`Gc` uses internal `Cell`, not thread-safe)

### Shell (`ish-shell/`)
- Single-threaded read-parse-execute loop
- `run_interactive()`: Reedline loop → `process_input()` → `ish_parser::parse()` → `vm.run()`
- `run_file()` / `run_inline()`: parse → `vm.run()` → exit
- Dependencies: reedline 0.46, nu-ansi-term 0.50

### Dependencies
- ish-vm: `gc 0.5`, `serde_json 1`, `ish-ast`
- ish-shell: `reedline 0.46`, `nu-ansi-term 0.50`, `ish-ast`, `ish-parser`, `ish-vm`, `ish-stdlib`
- No tokio dependency anywhere currently

### Acceptance Tests (`ish-tests/`)
- 255 tests across 7 groups (basics, functions, control_flow, error_handling, assurance_ledger, type_checking, type_narrowing)
- Pattern: `run_ish 'code'` → `assert_output "name" "expected" "$output"` → `finish`
- Run via `bash ish-tests/run_all.sh`

## Design Constraints

1. **GC is single-threaded.** All `Gc<>`, `Value`, `Environment` must stay on one thread. The `LocalSet`/`spawn_local` model satisfies this — all ish tasks share one thread.
2. **AST is Send.** `Program` contains only owned, `Send`-safe types. It can cross from the shell thread to the main thread.
3. **`&mut self` on IshVm.** The interpreter takes `&mut self` for defer_stack and ledger. With async, VM state is split into three objects: `IshVm` (shared), `TaskContext` (per-task, owns defer_stack), `YieldContext` (per-yield, owns yield budget). `IshVm` is wrapped in `Rc<RefCell<>>` for sharing across `spawn_local` tasks. Interpreter methods change from `&mut self` to take the three contexts explicitly.
4. **BuiltinFn uses `Rc`.** Builtins are `Rc<dyn Fn>` — NOT `Send`. This is fine for `spawn_local` but means builtins cannot be shared to parallel tasks.
5. **Existing tests must pass.** All 255 acceptance tests and all unit tests must continue to pass after the async migration.

## TODO

### Phase 1: Foundation — Tokio Runtime and Async Interpreter

Make the interpreter async without adding any new language features. All existing tests must pass.

- [x] 1. **Add tokio dependency** — `proto/ish-vm/Cargo.toml`, `proto/ish-shell/Cargo.toml`, `proto/Cargo.toml`
  - ish-vm: `tokio = { version = "1", features = ["rt", "time", "process", "sync"] }`
  - ish-shell: `tokio = { version = "1", features = ["rt-multi-thread", "macros"] }`
  - Workspace: add `[workspace.dependencies]` section for tokio version alignment

- [x] 2. **Make interpreter async and introduce three-context split** — `proto/ish-vm/src/interpreter.rs`
  - Split `IshVm` into three objects: `IshVm` (shared: global_env, ledger, builtins), `TaskContext` (per-task: defer_stack), `YieldContext` (per-yield: yield budget state)
  - Wrap `IshVm` in `Rc<RefCell<>>` for shared access across tasks
  - Change interpreter methods from `&mut self` to take `(vm: &Rc<RefCell<IshVm>>, task: &mut TaskContext, yc: &mut YieldContext)` (or similar)
  - Change `pub fn run(...)` to `pub async fn run(...)`
  - Change `fn exec_statement(...)` to `async fn exec_statement(...)`
  - Change `pub fn eval_expression(...)` to `pub async fn eval_expression(...)`
  - Change `pub fn call_function(...)` to `pub async fn call_function(...)`
  - Change `pop_and_run_defers()` to `async fn pop_and_run_defers(...)` (uses `TaskContext`)
  - Box async recursion where needed (async fn cannot be directly recursive — use `Box::pin`)
  - Add `.await` to all internal recursive calls (exec_statement, eval_expression, call_function)
  - Wire `ControlFlow` through async properly

- [x] 3. **Update builtin function signature** — `proto/ish-vm/src/value.rs`, `proto/ish-vm/src/builtins.rs`
  - Builtins remain synchronous (`Fn(&[Value]) -> Result<Value, RuntimeError>`)
  - In `call_function`, sync builtins are called directly (no `.await` needed)
  - No signature changes for existing builtins — they run synchronously on the `LocalSet` thread

- [x] 4. **Update shell to use Tokio runtime** — `proto/ish-shell/src/main.rs`, `proto/ish-shell/src/repl.rs`
  - Add `#[tokio::main]` to `main()` (or manually build `Runtime`)
  - In `run_inline()` and `run_file()`: wrap `vm.run()` in `LocalSet::run_until()`
  - In `run_interactive()`: for now, keep single-threaded — run `vm.run()` in `LocalSet` per submission
  - Two-thread model is deferred to Phase 5

- [x] 5. **Update dependent crates** — `proto/ish-stdlib/src/*.rs`, `proto/ish-codegen/src/*.rs`
  - `ish-stdlib`: `load_all()` calls `vm.run()` — must `.await` it. Add tokio dependency.
  - `ish-codegen`: Uses `vm.run()` and `vm.call_function()` — must `.await`. Add tokio dependency if needed, or defer (codegen is out of scope for concurrency impl).
  - All call sites to `vm.run()`, `vm.eval_expression()`, `vm.call_function()` must be updated

- [x] 6. **Update reflection** — `proto/ish-vm/src/reflection.rs`
  - `call_function` calls in reflection must `.await`

- [x] 7. **Verify existing tests pass** — `proto/ish-tests/run_all.sh`, `cargo test --workspace`
  - All 255 acceptance tests must pass
  - All unit tests must pass
  - No behavior change — just async wiring

### CHECKPOINT 1: All existing tests pass with async interpreter. No new features yet. `cargo build --workspace` succeeds. `cargo test --workspace` passes. `bash ish-tests/run_all.sh` passes.

### Phase 2: AST and Parser — New Concurrency Syntax

Add AST nodes and parser rules for async/await/spawn/yield.

- [x] 8. **Add async flag to FunctionDecl** — `proto/ish-ast/src/lib.rs`
  - Add `is_async: bool` field to `Statement::FunctionDecl`
  - Add `is_async: bool` field to `Expression::Lambda` (for async lambdas)
  - Update `ProgramBuilder` and display formatting if they reference FunctionDecl

- [x] 9. **Add new AST expression variants** — `proto/ish-ast/src/lib.rs`
  - `Expression::Await { expr: Box<Expression> }` — `await expr`
  - `Expression::Spawn { expr: Box<Expression> }` — `spawn expr`

- [x] 10. **Add new AST statement variants** — `proto/ish-ast/src/lib.rs`
  - `Statement::Yield` — explicit `yield` statement
  - `Statement::YieldEvery { count: Expression }` — `yield every N` (inside loops)
  - Update `ForEach` and `While` with optional `yield_every: Option<Expression>` field instead of a separate statement, if that better matches the syntax `for item in items yield every 500 { ... }`

- [x] 11. **Add yield-related annotation variants** — `proto/ish-ast/src/lib.rs`
  - `@[yield_budget(Xus)]` — handle via existing `Annotation::Entry` mechanism with `yield_budget` entry name and parameter
  - `@[unyielding]` — handle via existing `Annotation::Entry` mechanism

- [x] 12. **Update parser grammar** — `proto/ish-parser/src/ish.pest`
  - Add `async` keyword before `fn` in function declaration rule
  - Add `await` as a prefix unary expression (like `not`)
  - Add `spawn` as a prefix unary expression
  - Add `yield` as a standalone statement
  - Add `yield every` clause in `for`/`while` loops
  - Add `@[yield_budget(...)]` and `@[unyielding]` annotation parsing (may already be handled by generic annotation parsing)

- [x] 13. **Update parser builder** — `proto/ish-parser/src/builder.rs`
  - Map parsed `async fn` to `FunctionDecl { is_async: true, ... }`
  - Map parsed `await expr` to `Expression::Await { expr }`
  - Map parsed `spawn expr` to `Expression::Spawn { expr }`
  - Map parsed `yield` to `Statement::Yield`
  - Map parsed `yield every N` to loop field or `Statement::YieldEvery`

- [x] 14. **Update AST builder API** — `proto/ish-ast/src/builder.rs`
  - Existing `FunctionDecl` builder: set `is_async: false` default
  - Add builder methods for await, spawn, yield if needed for stdlib

- [x] 15. **Update AST display** — `proto/ish-ast/src/display.rs`
  - Display `async fn` prefix
  - Display `await expr`, `spawn expr`
  - Display `yield`, `yield every N`

- [x] 16. **Add parser tests** — `proto/ish-parser/tests/`
  - Parse `async fn foo() { }` → `FunctionDecl { is_async: true, ... }`
  - Parse `await some_call()` → `Expression::Await { ... }`
  - Parse `spawn some_call()` → `Expression::Spawn { ... }`
  - Parse `yield` → `Statement::Yield`
  - Parse `for x in items yield every 100 { ... }`
  - Parse incomplete: `await` alone → `Incomplete`
  - Parse incomplete: `spawn` alone → `Incomplete`
  - Existing parser tests must still pass

### CHECKPOINT 2: All new syntax parses correctly. All existing parser tests pass. `cargo test -p ish-parser` passes. AST round-trips correctly.

### Phase 3: Interpreter — Spawn, Await, Future Value

Implement `spawn`, `await`, and the `Future` value type.

- [x] 17. **Add Future value variant** — `proto/ish-vm/src/value.rs`
  - Add `Value::Future(FutureRef)` variant
  - `FutureRef` wraps `Rc<RefCell<Option<tokio::task::JoinHandle<Result<Value, RuntimeError>>>>>`
  - Use `Rc<RefCell<>>` (not `Gc`) since `JoinHandle` does not implement `Trace`
  - Implement `Trace` + `Finalize` with `#[unsafe_ignore_trace]`
  - Implement `Drop` for `FutureRef` to call `abort()` when the Future is dropped without being awaited (R1.7)
  - Update `to_display_string()` for `Value::Future` (e.g., `<future>`)
  - Update `type_of()` for `Value::Future` → `"future"`

- [x] 18. **Implement spawn** — `proto/ish-vm/src/interpreter.rs`
  - In `eval_expression`, handle `Expression::Spawn`:
    - The inner expression must be a function call
    - Clone `Rc<RefCell<IshVm>>` for the spawned task (cheap Rc clone)
    - Create a new `TaskContext` for the spawned task (fresh defer_stack)
    - Create a new `YieldContext` for the spawned task (fresh yield budget)
    - `tokio::task::spawn_local(async move { eval_expression(vm, &mut task_ctx, &mut yield_ctx, expr, env).await })`
    - Return `Value::Future` wrapping the `JoinHandle`

- [x] 19. **Implement await** — `proto/ish-vm/src/interpreter.rs`
  - In `eval_expression`, handle `Expression::Await`:
    - Evaluate the inner expression to get a `Value::Future`
    - Take the `JoinHandle` from the `FutureRef` (set inner to `None` to prevent double-await)
    - `.await` the `JoinHandle`
    - Handle `JoinError` (task panicked or was cancelled) — return cancellation error (E011)
    - Unwrap the `Result<Value, RuntimeError>` — propagate errors to the caller
  - Awaiting a non-future value at low assurance: return it directly (implicit identity await)

- [x] 20. **Implement yield** — `proto/ish-vm/src/interpreter.rs`
  - In `exec_statement`, handle `Statement::Yield`:
    - Call `tokio::task::yield_now().await`
    - Return `ControlFlow::None`

- [x] 21. **Add unit tests for spawn/await/Future** — `proto/ish-vm/src/interpreter.rs` (tests module)
  - Spawn a simple function, await the result
  - Await a non-future value (identity)
  - Drop a future without awaiting (verify task cancelled)
  - Await a cancelled future (verify cancellation error)
  - Error propagation through await

### CHECKPOINT 3: `spawn`, `await`, and `yield` work in the interpreter. Simple concurrent programs execute correctly. Existing tests still pass when run on the Tokio runtime.

### Phase 4: Yield Budget Mechanism

Implement automatic time-based yield insertion.

- [x] 22. **Add yield budget state to YieldContext** — `proto/ish-vm/src/interpreter.rs`
  - Define `YieldContext { budget_start: std::time::Instant, budget_duration: std::time::Duration, unyielding_depth: usize }`
  - Default `budget_duration` is 1ms
  - `YieldContext` is created fresh for each spawned task and reset on each yield
  - Helper: `async fn check_yield_budget(yc: &mut YieldContext)` — if elapsed > budget and unyielding_depth == 0, call `yield_now().await` and reset

- [x] 23. **Insert yield checks at yield-eligible points** — `proto/ish-vm/src/interpreter.rs`
  - Loop back-edges: In `While` and `ForEach` execution, call `check_yield_budget()` at each iteration
  - Function calls: In `call_function()`, call `check_yield_budget()` before executing the body
  - These are the "documented yield points" from the spec

- [x] 24. **Implement yield every N** — `proto/ish-vm/src/interpreter.rs`
  - In `ForEach` and `While` with `yield_every` field:
    - Maintain an iteration counter
    - Every N iterations, call `tokio::task::yield_now().await` regardless of time budget

- [x] 25. **Implement @[unyielding]** — `proto/ish-vm/src/interpreter.rs`
  - When executing an `Annotated` statement with `@[unyielding]`:
    - Increment `unyielding_depth`
    - Execute the inner statement
    - Decrement `unyielding_depth`
  - `check_yield_budget()` skips yielding when `unyielding_depth > 0`

- [x] 26. **Implement @[yield_budget(Xus)]** — `proto/ish-vm/src/interpreter.rs`
  - When executing an `Annotated` statement with `@[yield_budget(duration)]`:
    - Save current `yield_budget_duration`
    - Set new duration from the annotation parameter
    - Execute the inner statement
    - Restore previous duration

- [x] 27. **Add unit tests for yield budget** — `proto/ish-vm/src/interpreter.rs` (tests module)
  - Long-running loop yields control (verify via task interleaving)
  - `yield every 1` yields every iteration
  - `@[unyielding]` suppresses yielding
  - `@[yield_budget(100us)]` changes the threshold

### CHECKPOINT 4: Yield budget mechanism works. Long-running tasks yield cooperatively. `@[unyielding]` and `@[yield_budget]` work. Existing tests pass.

### Phase 5: Two-Thread Shell Architecture

Implement the two-thread model for interactive mode.

- [x] 28. **Implement two-thread interactive shell** — `proto/ish-shell/src/repl.rs`
  - Shell thread: Reedline loop → parse → send `Program` over `tokio::sync::mpsc` channel
  - Main thread: Tokio runtime with `LocalSet` → receive `Program` → `vm.run()` → send completion signal
  - Use `std::sync::mpsc` or `tokio::sync::mpsc` for shell→main (Program submission)
  - Use `tokio::sync::oneshot` for main→shell (completion signal per submission)
  - The shell thread is a regular OS thread (`std::thread::spawn`), not a Tokio task
  - The main thread runs `rt.block_on(local_set.run_until(recv_and_execute_loop()))`

- [x] 29. **Implement ExternalPrinter output routing** — `proto/ish-shell/src/repl.rs`, `proto/ish-vm/src/builtins.rs`
  - Create an `ExternalPrinter` from Reedline on the shell thread
  - Pass the sender channel to the VM constructor on the main thread
  - The `println` builtin captures `Option<Sender<String>>` in its closure at VM creation time: `Some(sender)` in interactive mode, `None` in non-interactive mode
  - When `Some(sender)`: send formatted string through channel to ExternalPrinter
  - When `None`: write to stdout directly (existing behavior)
  - Expression result display in `process_input` / the main thread loop must also route through ExternalPrinter

- [x] 30. **Update non-interactive modes** — `proto/ish-shell/src/repl.rs`
  - `run_file()` and `run_inline()`: single thread, `LocalSet`, `vm.run().await`, no ExternalPrinter
  - Output goes directly to stdout/stderr
  - Ensure Tokio runtime shuts down cleanly after program completes

- [x] 31. **Implement shell command async execution** — `proto/ish-vm/src/interpreter.rs`
  - Change `std::process::Command` to `tokio::process::Command` in `exec_shell_command`
  - Use `.output().await` instead of `.output()`
  - This requires exec_shell_command to be async (it should be, from Phase 1)

- [x] 32. **Implement future survival across submissions** — `proto/ish-shell/src/repl.rs`
  - Futures spawned in one shell submission survive in the `LocalSet`
  - The VM's `Environment` persists across submissions (already the case)
  - The `LocalSet` continues polling between submissions
  - Test: spawn a future in one submission, await it in the next

- [x] 33. **Add integration tests for two-thread model**
  - Interactive mode: spawn in one input, await in next
  - Background future output appears while shell is waiting
  - Parse errors display immediately (shell thread)
  - Runtime errors display via ExternalPrinter
  - Non-interactive mode: file execution with async code
  - Ctrl+D exits cleanly even with running futures

### CHECKPOINT 5: Two-thread shell works. Interactive spawn/await across submissions works. Output routing via ExternalPrinter works. Non-interactive modes work. All existing tests pass.

### Phase 6: Ledger Integration

Wire concurrency features into the assurance ledger.

- [x] 34. **Register Complexity and Yielding entry types** — `proto/ish-vm/src/ledger/entry_type.rs`
  - Register `Complexity` entry type: values `simple`/`complex`, applies_to `function`/`block`
  - Register `Yielding` entry type: values `yielding`/`unyielding`, applies_to `function`/`block`
  - (Parallel entry type deferred — no parallel functions yet)

- [x] 35. **Register concurrency assurance features** — `proto/ish-vm/src/ledger/standard.rs`
  - Register `async_annotation` feature: dimension `optional`/`required`
  - Register `await_required` feature: dimension `optional`/`required`
  - Register `guaranteed_yield` feature: dimension `disabled`/`enabled`
  - Register `yield_control` feature: dimension `time`/`time_and_count`
  - Register `future_drop` feature: dimension `disabled`/`enabled`
  - Update built-in standards: `streamlined` (all optional/disabled), `cautious` (required/enabled), `rigorous` (required/enabled + time_and_count)

- [x] 36. **Implement async_annotation audit** — `proto/ish-vm/src/ledger/audit.rs`, `proto/ish-vm/src/interpreter.rs`
  - When `async_annotation` is `required`: a function that performs `await` or `yield` without being declared `async fn` is a discrepancy
  - Audit at function declaration or at the point where a non-async function performs a yielding operation

- [x] 37. **Implement await_required audit** — `proto/ish-vm/src/ledger/audit.rs`, `proto/ish-vm/src/interpreter.rs`
  - When `await_required` is `required`: calling an async function without `await` is a discrepancy
  - At low assurance (optional): async calls are implicitly awaited

- [x] 38. **Implement future_drop audit** — `proto/ish-vm/src/interpreter.rs`
  - When `future_drop` is `enabled`: dropping a `Value::Future` without awaiting it triggers a discrepancy
  - Integrate with the `FutureRef` drop logic — check ledger state in the drop handler
  - Challenge: Drop handlers don't have access to the VM. May need a thread-local or `Rc<RefCell<LedgerState>>` reference.

- [x] 39. **Implement guaranteed_yield audit** — `proto/ish-vm/src/ledger/audit.rs`
  - When `guaranteed_yield` is `enabled`: a function or block that is `complex + unyielding` is a discrepancy
  - This requires Complexity and Yielding entries to be inferred or declared on functions

- [x] 40. **Add ledger integration tests** — `proto/ish-tests/assurance_ledger/`
  - `async_annotation` required: non-async yielding function → discrepancy
  - `await_required` required: unawaited async call → discrepancy
  - `future_drop` enabled: dropped future → discrepancy
  - `guaranteed_yield` enabled: complex + unyielding → discrepancy
  - All features at low assurance → no discrepancies

### CHECKPOINT 6: Ledger integration works. Concurrency features produce correct discrepancies at appropriate assurance levels. Existing ledger tests pass.

### Phase 7: Acceptance Tests

Comprehensive acceptance tests for all concurrency features.

- [x] 41. **Create concurrency test group** — `proto/ish-tests/concurrency/`
  - Create `run_group.sh` that discovers and runs all test files in the group
  - Register the group in `run_all.sh`

- [x] 42. **Add basic async/await tests** — `proto/ish-tests/concurrency/async_await.sh`
  - `async fn` declaration and call
  - `await` on a spawned future
  - `spawn` returns a future
  - Multiple spawn + await (interleaving)
  - Await a non-future value (identity at low assurance)

- [x] 43. **Add cancellation tests** — `proto/ish-tests/concurrency/cancellation.sh`
  - Drop a future → task cancelled
  - Await a cancelled future → cancellation error (catchable)
  - Defer runs in cancelled task
  - Error in spawned task + dropped future → error logged

- [x] 44. **Add yield tests** — `proto/ish-tests/concurrency/yield_control.sh`
  - Explicit `yield` statement
  - `yield every N` in for loop
  - `@[unyielding]` suppresses yield
  - `@[yield_budget(Xus)]` changes threshold
  - Long-running task yields control to other tasks

- [x] 45. **Add error propagation tests** — `proto/ish-tests/concurrency/error_propagation.sh`
  - Error in awaited function propagates to caller
  - Error in spawned function captured in future, re-thrown on await
  - try/catch around await catches errors
  - defer runs even when spawned task errors

- [x] 46. **Add shell integration tests** — `proto/ish-tests/concurrency/shell_integration.sh`
  - Shell commands work in async context
  - Shell command via `tokio::process::Command`
  - Background future output appears
  - println from spawned task appears

- [x] 47. **Add assurance level tests** — `proto/ish-tests/concurrency/assurance_levels.sh`
  - Low assurance: async keywords not required, implicit await
  - High assurance: async/await required, discrepancies on missing annotations
  - future_drop enabled: dropped future → discrepancy

### CHECKPOINT 7: All concurrency acceptance tests pass. All existing 255 acceptance tests pass. All unit tests pass. `cargo test --workspace` clean. `bash ish-tests/run_all.sh` clean.

### Phase 8: Finalization

- [x] 48. **Update roadmap** — `docs/project/roadmap.md`
  - Move "Concurrency prototype" to Completed (or add new item for prototype impl)

- [x] 49. **Update maturity matrix** — `docs/project/maturity.md`
  - Update Concurrency row: Prototyped=✅, Tested=partial

- [x] 50. **Update plans index** — `docs/project/plans/INDEX.md`
  - Add row for this plan

### CHECKPOINT 8: Final verification. `cargo build --workspace` clean. `cargo test --workspace` passes. `bash ish-tests/run_all.sh` passes. Roadmap and maturity updated.

## Reference

### Decisions Summary (from proposal)

| # | Key point for implementation |
|---|---------------------------|
| 1 | Three dimensions: Complexity, Yielding, Blocking (eliminated) |
| 3 | Low assurance: implicit await, no async keyword required |
| 4 | Standard async/await, no structured concurrency |
| 5 | Hybrid yield: time-based default, statement-count at high assurance |
| 6 | Tokio `LocalSet`/`spawn_local`. `tokio::spawn` for parallel (Rust only) |
| 9 | Blocking eliminated. All I/O async via Tokio |
| 10 | `future_drop` discrepancy when enabled |
| 12 | `spawn` does NOT make caller yielding |
| 16 | `future_drop` is enabled/disabled toggle |
| 20 | Only `println` for I/O. All other I/O deferred |
| 21 | Two threads interactive. Shell thread + main/VM thread |
| 22 | Parser on shell thread. AST is `Send` |
| 23 | ExternalPrinter for output. No result channel to shell |

### Key Architectural Decisions for Implementation

**VM ownership model (decided):** VM state is split into three objects with different cardinalities and lifetimes:

1. **`IshVm`** — shared state (global env, ledger, builtins). One per interpreter. Wrapped in `Rc<RefCell<>>` where needed for borrowing across `spawn_local` tasks.
2. **`TaskContext`** — per-task state (defer_stack). One per spawned task. Each task creates its own `TaskContext` referencing the shared `IshVm`.
3. **`YieldContext`** — yield budget state (yield timer, budget duration, unyielding depth). Reset every time the VM yields.

All three live on the same `LocalSet` thread. Use `Rc<RefCell<>>` wrapping where borrowing issues remain.

**BuiltinFn for println routing (decided):** Currently `BuiltinFn` is `Fn(&[Value]) -> Result<Value, RuntimeError>`. For println to route through ExternalPrinter, it needs access to the output channel. **Decision:** Capture the channel in the builtin closure at VM creation time. The `println` builtin closure captures an `Option<Sender<String>>` when the VM is constructed — `Some(sender)` in interactive mode (routes to ExternalPrinter), `None` in non-interactive mode (writes to stdout).


### Existing File Sizes (for effort estimation)

| File | Lines |
|------|-------|
| `ish-vm/src/interpreter.rs` | 2,951 |
| `ish-vm/src/builtins.rs` | 840 |
| `ish-vm/src/value.rs` | 206 |
| `ish-vm/src/environment.rs` | 76 |
| `ish-shell/src/repl.rs` | ~150 |
| `ish-shell/src/main.rs` | ~30 |
| `ish-ast/src/lib.rs` | ~500 |

The interpreter (2,951 lines) is the most affected file. Every `eval_expression` and `exec_statement` call becomes `.await`.

---

## Referenced by

- [docs/project/plans/INDEX.md](INDEX.md)
- [docs/project/proposals/concurrency.md](../proposals/concurrency.md)
- [docs/project/plans/concurrency.md](concurrency.md)
