---
title: "Plan: Concurrency Correctness Fixes"
category: plan
audience: [ai-dev]
status: completed
last-verified: 2026-04-02
depends-on:
  - docs/project/proposals/concurrency-correctness.md
  - docs/spec/concurrency.md
  - docs/spec/types.md
  - docs/spec/errors.md
  - docs/architecture/vm.md
  - docs/architecture/ast.md
---

# Plan: Concurrency Correctness Fixes

*Derived from [concurrency-correctness.md](../proposals/concurrency-correctness.md) on 2026-04-02.*

## Overview

Implement three concurrency correctness fixes: (1) FutureRef identity equality via `Rc::ptr_eq`, (2) grammar-level restriction of `await`/`spawn` to function calls with E012/E013 validation, and (3) a compiled function architecture that unifies builtins with user-defined functions through a `FunctionImplementation` enum, with implied await at low assurance for parallel builtins.

## Requirements

Extracted from the accepted proposal. Each is a testable statement.

### Feature 1 — FutureRef Identity Equality

- R1.1: `Value::Future` equality uses `Rc::ptr_eq` on the inner `Rc`.
- R1.2: `f == f` returns `true` for a future value `f`.
- R1.3: Two independently created futures are not equal.
- R1.4: A cloned future reference is equal to the original.

### Feature 2 — Grammar-Level Spawn/Await Restriction

- R2.1: The grammar rule for `await` takes `call_expr` (not `unary`).
- R2.2: The grammar rule for `spawn` takes `call_expr` (not `unary`).
- R2.3: `await 42` produces an `Incomplete` AST node, not a valid `Await` node.
- R2.4: `spawn "hello"` produces an `Incomplete` AST node, not a valid `Spawn` node.
- R2.5: AST `Await` node contains `callee: Box<Expression>` and `args: Vec<Expression>`.
- R2.6: AST `Spawn` node contains `callee: Box<Expression>` and `args: Vec<Expression>`.
- R2.7: `await unyielding_fn()` throws E012 before calling the function.
- R2.8: `spawn unyielding_fn()` throws E013 before calling the function.
- R2.9: `await ambiguous_fn()` where the function has no Yielding entry and returns a non-Future passes through.
- R2.10: `await yielding_fn()` where the result is a Future performs a normal await.

### Feature 3 — Compiled Function Implementation

- R3.1: `FunctionImplementation` enum has exactly two variants: `Interpreted(Statement)` and `Compiled(Shim)`.
- R3.2: `Shim` type is `Rc<dyn Fn(&[Value]) -> Result<Value, RuntimeError>>` (synchronous).
- R3.3: `IshFunction` has `implementation: FunctionImplementation` instead of `body: Statement`.
- R3.4: `IshFunction` has `has_yielding_entry: Option<bool>`.
- R3.5: `BuiltinFn` struct is eliminated.
- R3.6: `Value::BuiltinFunction` variant is eliminated.
- R3.7: All builtins are registered as `Value::Function` with `Compiled` implementations.
- R3.8: `call_function_inner` dispatches `Compiled(shim)` by calling `shim(args)` synchronously.
- R3.9: Ledger builtins are intercepted by name before shim dispatch.
- R3.10: Unyielding builtins (e.g., `len`) return a plain `Value`.
- R3.11: Parallel builtins (`print`, `println`, `read_file`, `write_file`) return `Value::Future`.
- R3.12: At low assurance (no `await_required`), bare calls to parallel builtins imply `await` — the interpreter awaits the future and returns the resolved value.
- R3.13: At high assurance (`await_required` is `required`), bare calls to parallel builtins return the `Future` — unawaited-future audit catches it.
- R3.14: Builtins have parameter metadata (names, types) and return type metadata.
- R3.15: Arity checking applies to builtins (E003 for wrong arg count).
- R3.16: `println("hello")` continues to work without `await` at low assurance (backward compatible).
- R3.17: Parallel builtin futures do not resolve until I/O is complete.

## Authority Order

1. GLOSSARY.md (new terms)
2. Roadmap (set to "in progress")
3. Specification docs
4. Architecture docs
5. User guide / AI guide
6. Agent documentation
7. Acceptance tests
8. Code (implementation)
9. Unit tests
10. Roadmap (set to "completed")
11. History
12. Index files

## TODO

### Phase 0 — Glossary and Roadmap

- [x] 1. **Add glossary terms** — `GLOSSARY.md`
  - Add "compiled function": An `IshFunction` with a `Compiled(Shim)` implementation. Builtins and future stdlib functions are compiled functions.
  - Add "implied await": When `await_required` is not active, the interpreter automatically awaits a `Future` returned by a bare function call. Makes parallel builtins backward-compatible.
  - Add "function implementation": The `FunctionImplementation` enum that distinguishes `Interpreted` (tree-walking) from `Compiled` (shim) function execution.
  - Update "shim function": Expand to cover compiled builtin shims (synchronous `Fn(&[Value]) -> Result<Value, RuntimeError>`).
  - Update "parallel shim": Update to reflect the unified Shim type (no longer a separate variant).

- [x] 2. **Update roadmap** — `docs/project/roadmap.md`
  - Add "Concurrency correctness fixes (FutureRef equality, grammar-level await/spawn, compiled functions)" to "In Progress" section.

### Phase 1 — Specification Docs

- [x] 3. **Update concurrency spec** — `docs/spec/concurrency.md`
  - Add: grammar restriction on `await`/`spawn` to function calls.
  - Add: E012 for awaiting explicitly unyielding functions.
  - Add: E013 for spawning explicitly unyielding functions.
  - Add: ambiguous pass-through (no Yielding entry, non-Future result).
  - Add: Future identity equality (`Rc::ptr_eq`).
  - Add: parallel builtins with implied await at low assurance.

- [x] 4. **Update types spec** — `docs/spec/types.md`
  - Add: `Future<T>` uses identity equality (`Rc::ptr_eq`).

- [x] 5. **Update syntax spec** — `docs/spec/syntax.md`
  - Update: `await` and `spawn` grammar rules take `call_expr` not `unary`.

- [x] 6. **Update error spec and reconcile E011/E012** — `docs/spec/errors.md`
  - Reconcile existing E011 ("Cancellation error") with proposal E011 ("ConcurrencyError" — broadened to include cancelled, panicked, assurance discrepancy, already-awaited). E011 already serves this role in the code (8 usage sites).
  - Reconcile existing E012 ("Future dropped without being awaited") — this is a subset of E011 (assurance discrepancy). Merge into E011 description. Reassign E012 to "await type mismatch".
  - Add E013: spawn type mismatch.
  - Update the error codes table accordingly.

- [x] 7. **Update error catalog** — `docs/errors/INDEX.md`
  - Add E011 row: `ConcurrencyError` — Concurrency error (cancelled task, panicked task, assurance discrepancy, already-awaited future).
  - Update E012 row: `TypeError` — Await type mismatch (`await` applied to a call to an explicitly unyielding function).
  - Add E013 row: `ConcurrencyError` — Spawn type mismatch (`spawn` applied to a call to an explicitly unyielding function).

**Checkpoint A**: All specification docs updated. Review for internal consistency.

### Phase 2 — Architecture Docs

- [x] 8. **Update VM architecture** — `docs/architecture/vm.md`
  - Document `FunctionImplementation` enum (Interpreted/Compiled).
  - Document elimination of `BuiltinFn`/`Value::BuiltinFunction`.
  - Document unified function dispatch in `call_function_inner`.
  - Document shim types (unyielding, yielding, parallel).
  - Document implied await logic.
  - Document ledger builtin name-based interception.

- [x] 9. **Update AST architecture** — `docs/architecture/ast.md`
  - Document `Await` and `Spawn` AST nodes with `callee + args` instead of generic expression.

### Phase 3 — Guide Docs

- [x] 10. **Update concurrency playbook** — `docs/ai-guide/playbook-concurrency.md`
  - Add: grammar-level await/spawn restriction (must be a function call).
  - Add: parallel builtins with implied await at low assurance.
  - Add: E012/E013 for unyielding functions.

- [x] 11. **Update antipatterns** — `docs/ai-guide/antipatterns.md`
  - Add: `await 42` — parse error (Incomplete node).
  - Add: `spawn unyielding_fn()` — E013.
  - Add: `await unyielding_fn()` — E012.

- [x] 12. **Update open questions** — `docs/project/open-questions.md`
  - Add Concurrency section with two open questions:
    1. Function yielding categorization at declaration time.
    2. Builtin replacement by standard library.

**Checkpoint B**: All documentation updated. Review for consistency with spec.

### Phase 4 — Acceptance Tests

- [x] 13. **Write acceptance tests for Feature 1** — `proto/ish-tests/concurrency/future_equality.sh` (new file)
  - Test: `f == f` → true for a future.
  - Test: two independent futures → not equal.
  - Test: cloned future reference → equal.

- [x] 14. **Write acceptance tests for Feature 2 — grammar restriction** — update `proto/ish-tests/concurrency/spawn_await.sh`
  - Test: `await 42` → parse error.
  - Test: `await "hello"` → parse error.
  - Test: `spawn 42` → parse error.
  - Test: `spawn "hello"` → parse error.
  - Test: `await unyielding_fn()` → E012.
  - Test: `spawn unyielding_fn()` → E013.
  - Test: `await ambiguous_fn()` → pass-through.
  - Update any existing tests that used `await <non-call>` to use the new grammar.

- [x] 15. **Write acceptance tests for Feature 3 — compiled functions** — `proto/ish-tests/concurrency/compiled_functions.sh` (new file)
  - Test: `println("hello")` works without await (implied await, low assurance).
  - Test: builtins report correct arity errors (E003).
  - Test: `await len([1,2])` → E012.
  - Test: `spawn len([1,2])` → E013.
  - Test: `type_of(println)` → "function" (not "builtin").

**Checkpoint C**: All acceptance tests written. They should fail (code not yet changed).

### Phase 5 — Code: Feature 1 (FutureRef Equality)

- [x] 16. **Fix FutureRef PartialEq** — `proto/ish-vm/src/value.rs`
  - Change `(Value::Future(_), Value::Future(_)) => false` to `(Value::Future(a), Value::Future(b)) => Rc::ptr_eq(&a.inner, &b.inner)`.
  - This requires making `inner` accessible (currently private). Either add a public accessor method or change visibility.

- [x] 17. **Run tests** — `cd proto && cargo test --workspace && bash ish-tests/run_all.sh`
  - Verify Feature 1 acceptance tests pass.
  - Verify no regressions.

**Checkpoint D**: Feature 1 complete. FutureRef equality tests pass.

### Phase 6 — Code: Feature 2 (Grammar Restriction)

- [x] 18. **Update PEG grammar** — `proto/ish-parser/src/ish.pest`
  - Change `await_op ~ unary` to `await_op ~ call_expr`.
  - Change `spawn_op ~ unary` to `spawn_op ~ call_expr`.
  - Add `call_expr` rule: `call_expr = { primary ~ call_args }`.

- [x] 19. **Update AST definition** — `proto/ish-ast/src/lib.rs`
  - Change `Expression::Await { expr: Box<Expression> }` to `Expression::Await { callee: Box<Expression>, args: Vec<Expression> }`.
  - Change `Expression::Spawn { expr: Box<Expression> }` to `Expression::Spawn { callee: Box<Expression>, args: Vec<Expression> }`.

- [x] 20. **Update AST builder** — `proto/ish-parser/src/ast_builder.rs`
  - Update `Rule::await_op` handler to build `Await { callee, args }` from a `call_expr` pair.
  - Update `Rule::spawn_op` handler to build `Spawn { callee, args }` from a `call_expr` pair.
  - Add fallback to `Incomplete` node when `await`/`spawn` is followed by a non-call.

- [x] 21. **Update AST Display/Debug** — `proto/ish-ast/src/lib.rs` (or display impl file)
  - Update `Display` for `Await` and `Spawn` to format `callee(args)`.

- [x] 22. **Update interpreter Await handler** — `proto/ish-vm/src/interpreter.rs` (lines ~974-1012)
  - Destructure `Await { callee, args }` instead of `Await { expr }`.
  - Resolve callee, check `has_yielding_entry`, throw E012 if `Some(false)`.
  - Evaluate args, call function, handle Future result or pass-through.

- [x] 23. **Update interpreter Spawn handler** — `proto/ish-vm/src/interpreter.rs` (lines ~1020-1035)
  - Destructure `Spawn { callee, args }` instead of `Spawn { expr }`.
  - Resolve callee, check `has_yielding_entry`, throw E013 if `Some(false)`.
  - Evaluate args, spawn task with `call_function_inner`.

- [x] 24. **Fix all compilation errors** from AST changes — multiple files
  - Any code matching on `Expression::Await { expr }` or `Expression::Spawn { expr }` must be updated.
  - Search all crates for these patterns: `ish-ast`, `ish-parser`, `ish-vm`, `ish-stdlib`, `ish-codegen`.

- [x] 25. **Run build** — `cd proto && cargo build --workspace`
  - Fix any remaining compilation errors.

- [x] 26. **Run tests** — `cd proto && cargo test --workspace && bash ish-tests/run_all.sh`
  - Note: Feature 2 acceptance tests for E012/E013 on unyielding functions will still fail (Feature 3 adds `has_yielding_entry`). Grammar-level tests (await non-call → error) should pass.

**Checkpoint E**: Feature 2 grammar changes complete. `await`/`spawn` require call expressions. Parse-level tests pass.

### Phase 7 — Code: Feature 3 (Compiled Functions)

#### Phase 7a — Type System Changes

- [x] 27. **Add `FunctionImplementation` enum** — `proto/ish-vm/src/value.rs`
  - Add `FunctionImplementation` enum with `Interpreted(Statement)` and `Compiled(Shim)` variants.
  - Add `Shim` type alias: `pub type Shim = Rc<dyn Fn(&[Value]) -> Result<Value, RuntimeError>>`.
  - Implement `Clone`, `Debug` for `FunctionImplementation`.

- [x] 28. **Update `IshFunction`** — `proto/ish-vm/src/value.rs`
  - Replace `body: Statement` with `implementation: FunctionImplementation`.
  - Add `has_yielding_entry: Option<bool>`.
  - Update `new_function` constructor to accept `FunctionImplementation` and `has_yielding_entry`.
  - Existing callers of `new_function` pass `FunctionImplementation::Interpreted(body)` and `None` for `has_yielding_entry`.

- [x] 29. **Add `new_compiled_function` constructor** — `proto/ish-vm/src/value.rs`
  - Helper that creates a `Value::Function` with `Compiled(shim)` implementation and appropriate metadata.
  - Takes: name, params (with types), return_type, shim, has_yielding_entry, closure_env.

- [x] 30. **Remove `BuiltinFn`, `BuiltinRef`, `new_builtin`** — `proto/ish-vm/src/value.rs`
  - Remove `BuiltinFn` struct.
  - Remove `BuiltinRef` type alias.
  - Remove `new_builtin` function.
  - Remove `Value::BuiltinFunction` variant.

- [x] 31. **Fix compilation errors from Value enum change** — multiple files
  - Every match on `Value::BuiltinFunction` must be removed or converted. Search all crates.
  - Every call to `new_builtin()` must be converted to `new_compiled_function()`.
  - Every reference to `IshFunction.body` must become `IshFunction.implementation`.
  - Every reference to `BuiltinFn` must be removed.

- [x] 32. **Run build** — `cd proto && cargo build --workspace`
  - This is expected to fail until builtins.rs and interpreter.rs are updated. Address compilation errors iteratively.

#### Phase 7b — Interpreter Changes

- [x] 33. **Update `call_function_inner`** — `proto/ish-vm/src/interpreter.rs` (lines ~1274-1365)
  - Remove `Value::BuiltinFunction` match arm.
  - In `Value::Function` arm: after arity check and param audit, add ledger builtin name interception (before dispatch).
  - Add dispatch: `FunctionImplementation::Interpreted(body)` → existing logic (create env, bind params, exec_statement).
  - Add dispatch: `FunctionImplementation::Compiled(shim)` → `shim(args)`.
  - Update the `Interpreted` arm to get body from `f.implementation` instead of `f.body`.

- [x] 34. **Add implied await to FunctionCall handler** — `proto/ish-vm/src/interpreter.rs` (lines ~846-852)
  - After `call_function_inner` returns, check if result is `Value::Future`.
  - If `await_required` is not active, immediately await the future (implied await).
  - If `await_required` is active, return the Future as-is.

- [x] 35. **Add `has_unyielding_entry` helper** — `proto/ish-vm/src/interpreter.rs`
  - Function that checks `IshFunction.has_yielding_entry == Some(false)`.
  - Used by Await handler (E012) and Spawn handler (E013).

- [x] 36. **Update Await and Spawn handlers for E012/E013** — `proto/ish-vm/src/interpreter.rs`
  - The Await handler (TODO 22) already has the check structure from Phase 6. Now that `has_yielding_entry` exists, the check has data to operate on.
  - Verify the Spawn handler (TODO 23) similarly uses `has_yielding_entry`.

#### Phase 7c — Builtins Rewrite

- [x] 37. **Update `BuiltinConfig`** — `proto/ish-vm/src/builtins.rs`
  - Keep the struct as-is (output_sender). It becomes the configuration for shim closures.

- [x] 38. **Rewrite `register_all_with_config`** — `proto/ish-vm/src/builtins.rs`
  - Same structure (register_io, register_strings, etc.) but all use `new_compiled_function` instead of `new_builtin`.

- [x] 39. **Rewrite I/O builtins as parallel shims** — `proto/ish-vm/src/builtins.rs`
  - `print`: Compiled shim with `has_yielding_entry: Some(true)`. Interactive mode: sends output + oneshot ack channel. Non-interactive: `spawn_blocking` for stdout + `spawn_local` bridge. Returns `Value::Future`.
  - `println`: Same pattern as `print`.
  - `read_file`: Compiled shim. `spawn_blocking` for `fs::read_to_string` + `spawn_local` bridge. Returns `Value::Future`.
  - `write_file`: Compiled shim. `spawn_blocking` for `fs::write` + `spawn_local` bridge. Returns `Value::Future`.
  - All four have `has_yielding_entry: Some(true)`.

- [x] 40. **Rewrite simple builtins as unyielding shims** — `proto/ish-vm/src/builtins.rs`
  - Convert all string, list, object, type, conversion, and error builtins to `new_compiled_function` with `has_yielding_entry: Some(false)`.
  - Add parameter metadata (names and optionally types) and return type metadata.
  - Logic remains the same — just wrap in `Compiled(Shim)` instead of `BuiltinFn`.

- [x] 41. **Rewrite ledger builtins** — `proto/ish-vm/src/builtins.rs`
  - Register as `new_compiled_function` with stub shim (panics if called directly — intercepted by name in `call_function_inner`).
  - Add parameter metadata and `has_yielding_entry: Some(false)`.

- [x] 42. **Update `BuiltinConfig` for I/O completion** — `proto/ish-vm/src/builtins.rs`
  - Change `output_sender: Option<crossbeam::channel::Sender<String>>` to carry a type that includes oneshot ack capability. Options:
    - Change to `Sender<(String, Option<tokio::sync::oneshot::Sender<()>>)>`, or
    - Keep `Sender<String>` and accept that interactive mode I/O completion is deferred (pragmatic for prototype).
  - Decision: If oneshot ack adds significant complexity (crossbeam is `Send`, tokio oneshot is `Send`), defer full I/O completion to a follow-up. Document the limitation.

- [x] 43. **Update shell integration** — `proto/ish-shell/src/main.rs` (or wherever shell reads from the channel)
  - If ack channel was added in TODO 42, the shell reader must send the ack after writing output.
  - If deferred, no change needed.

- [x] 44. **Update AST reflection builtins** — `proto/ish-vm/src/reflection.rs`
  - Convert `register_ast_builtins` to use `new_compiled_function` instead of `new_builtin`.

- [x] 45. **Run build** — `cd proto && cargo build --workspace`
  - Fix any remaining compilation errors.

- [x] 46. **Run tests** — `cd proto && cargo test --workspace && bash ish-tests/run_all.sh`
  - All acceptance tests should pass.
  - All unit tests should pass.

**Checkpoint F**: Feature 3 complete. All compiled function tests pass. Implied await works.

### Phase 8 — Unit Tests

- [x] 47. **Add unit tests for FutureRef equality** — `proto/ish-vm/src/value.rs` (test module)
  - Test `Rc::ptr_eq` behavior for same/different/cloned futures.

- [x] 48. **Add unit tests for FunctionImplementation** — `proto/ish-vm/src/value.rs` (test module)
  - Test `new_compiled_function` creates correct Value::Function.
  - Test `has_yielding_entry` is set correctly.

- [x] 49. **Add parser tests for grammar restriction** — `proto/ish-parser/tests/`
  - Test: `await foo()` parses to `Await { callee: Identifier("foo"), args: [] }`.
  - Test: `spawn bar(x)` parses to `Spawn { callee: Identifier("bar"), args: [Identifier("x")] }`.
  - Test: `await 42` parses to `Incomplete` node.
  - Test: `spawn "hello"` parses to `Incomplete` node.

- [x] 50. **Run full test suite** — `cd proto && cargo test --workspace && bash ish-tests/run_all.sh`

**Checkpoint G**: All unit tests pass. Full regression suite green.

### Phase 9 — Finalize

- [x] 51. **Update roadmap** — `docs/project/roadmap.md`
  - Move "Concurrency correctness fixes" from "In Progress" to "Completed".

- [x] 52. **Update history** — `docs/project/history/INDEX.md`
  - Verify `2026-04-01-concurrency-correctness` entry exists.

- [x] 53. **Update maturity matrix** — `docs/project/maturity.md`
  - Update rows affected by these changes (concurrency, types, builtins).

- [x] 54. **Update index files** — `docs/INDEX.md`, `docs/errors/INDEX.md`, `docs/spec/INDEX.md`
  - Verify all new/changed files are indexed.

- [x] 55. **Final test run** — `cd proto && cargo test --workspace && bash ish-tests/run_all.sh`

**Checkpoint H**: All artifacts updated. Full test suite green. Implementation complete.

## Reference

### Error Code Reconciliation

The existing codebase uses E011 in 8 places in `interpreter.rs` for various concurrency errors (cancellation, panic, future drop, assurance discrepancy, already-awaited). The proposal broadens E011 to "ConcurrencyError" as a catch-all — this is consistent with existing usage.

The existing `docs/spec/errors.md` defines E012 as "Future dropped without being awaited" — but E012 is **not used anywhere in the code**. E011 handles the future-drop case (lines 193, 705 in interpreter.rs). The proposal reassigns E012 to "await type mismatch", which is safe.

### Current Source Locations

| Item | File | Lines |
|------|------|-------|
| `Value` enum | `proto/ish-vm/src/value.rs` | 48-53 |
| `FutureRef` struct | `proto/ish-vm/src/value.rs` | 156-178 |
| `BuiltinFn` struct | `proto/ish-vm/src/value.rs` | 130-145 |
| `IshFunction` struct | `proto/ish-vm/src/value.rs` | 82-98 |
| `new_function` | `proto/ish-vm/src/value.rs` | 105-120 |
| `new_builtin` | `proto/ish-vm/src/value.rs` | 147-154 |
| `PartialEq for Value` (Future arm) | `proto/ish-vm/src/value.rs` | ~261-275 |
| `Expression::Await` | `proto/ish-ast/src/lib.rs` | 219-221 |
| `Expression::Spawn` | `proto/ish-ast/src/lib.rs` | 222-224 |
| `Expression::FunctionCall` | `proto/ish-ast/src/lib.rs` | 205-208 |
| PEG `unary` rule | `proto/ish-parser/src/ish.pest` | 187-192 |
| PEG `call_args` rule | `proto/ish-parser/src/ish.pest` | 205 |
| PEG `postfix` rule | `proto/ish-parser/src/ish.pest` | 199-207 |
| AST builder Await | `proto/ish-parser/src/ast_builder.rs` | 1020-1025 |
| AST builder Spawn | `proto/ish-parser/src/ast_builder.rs` | 1026-1031 |
| Interpreter Await handler | `proto/ish-vm/src/interpreter.rs` | 974-1012 |
| Interpreter Spawn handler | `proto/ish-vm/src/interpreter.rs` | 1020-1035 |
| Interpreter FunctionCall handler | `proto/ish-vm/src/interpreter.rs` | 846-852 |
| `call_function_inner` | `proto/ish-vm/src/interpreter.rs` | 1274-1365 |
| Builtin dispatch (name match) | `proto/ish-vm/src/interpreter.rs` | 1331-1337 |
| `IshVm::new` (builtin registration) | `proto/ish-vm/src/interpreter.rs` | 121-128 |
| `register_all` / `register_all_with_config` | `proto/ish-vm/src/builtins.rs` | 20-32 |
| `BuiltinConfig` | `proto/ish-vm/src/builtins.rs` | 12-17 |
| Ledger builtin stubs | `proto/ish-vm/src/builtins.rs` | 42-65 |
| I/O builtins (print, println, read_file, write_file) | `proto/ish-vm/src/builtins.rs` | 67-140 |
| Error catalog | `docs/errors/INDEX.md` | 26-35 |
| Error spec table | `docs/spec/errors.md` | 212-223 |

### Glossary Terms to Add/Update

| Action | Term | Definition |
|--------|------|-----------|
| Add | compiled function | An `IshFunction` with a `Compiled(Shim)` implementation instead of an `Interpreted(Statement)` body. Builtins and future stdlib functions are compiled functions. |
| Add | implied await | When the `await_required` assurance feature is not active, the interpreter automatically awaits a `Future` returned by a bare function call (no explicit `await`/`spawn`). Preserves backward compatibility for parallel builtins at low assurance. |
| Add | function implementation | The execution strategy for a function, described by the `FunctionImplementation` enum: `Interpreted(Statement)` for tree-walking execution, or `Compiled(Shim)` for synchronous shim dispatch. |
| Update | shim function | Expand: In the compiled function architecture, a shim is a synchronous Rust closure (`Fn(&[Value]) -> Result<Value, RuntimeError>`) that receives already-validated arguments, performs the function's work (or spawns it), and returns a `Value` (which may be `Value::Future` for yielding/parallel functions). |
| Update | parallel shim | Update: A shim for a parallel builtin. Marshals arguments into `Send`-safe form, spawns work via `spawn_blocking`, uses a `spawn_local` bridge to convert native results back to `Value`, and returns `Value::Future`. Part of the unified `Shim` type. |

### Test Counts (Pre-Implementation)

- Unit tests: 337
- Acceptance tests: ~300 (from `ish-tests/run_all.sh`)

## Referenced by

- [docs/project/plans/INDEX.md](INDEX.md)
- [docs/project/proposals/concurrency-correctness.md](../proposals/concurrency-correctness.md)
