---
title: "Plan: Stubbed Analyzer and Yielding/Unyielding Function Refactoring"
category: plan
audience: [ai-dev]
status: completed
last-verified: 2026-04-03
depends-on:
  - docs/project/proposals/stubbed-analyzer.md
  - docs/spec/concurrency.md
  - docs/architecture/vm.md
  - proto/ish-vm/src/interpreter.rs
  - proto/ish-runtime/src/value.rs
---

# Plan: Stubbed Analyzer and Yielding/Unyielding Function Refactoring

*Derived from [stubbed-analyzer.md](../proposals/stubbed-analyzer.md) on 2026-04-03.*

## Overview

Remove the defective `PENDING_INTERP_CALL` thread-local mechanism from the interpreter. Replace it with self-contained shims that execute function bodies directly — yielding shims spawn a `tokio::task::spawn_local` and return `Value::Future`, unyielding shims execute synchronously and return the value. This requires a stubbed code analyzer to classify functions at declaration time, splitting the interpreter into yielding/unyielding execution paths, and making `call_function_inner` synchronous.

## Requirements

Extracted from the accepted design proposal. Each requirement is a testable statement.

- R1: A new `analyzer.rs` module exists in `ish-vm` that classifies functions as yielding or unyielding.
- R2: The analyzer classifies `async fn` as yielding without walking the body.
- R3: The analyzer classifies functions containing `await`, `spawn`, `yield`, or `$()` as yielding.
- R4: The analyzer classifies functions that call yielding functions as yielding (propagation via implied await).
- R5: The analyzer errors on calls to undefined functions (no forward references in stub).
- R6: The analyzer does not recurse into nested function declarations or lambdas.
- R7: `has_yielding_entry: None` is treated as unyielding everywhere.
- R8: All newly declared functions and lambdas have `has_yielding_entry` set to `Some(true)` or `Some(false)`.
- R9: `exec_statement_unyielding` and `eval_expression_unyielding` are synchronous (no `Pin<Box<dyn Future>>`, no `.await`).
- R10: `exec_statement_yielding` and `eval_expression_yielding` are async (current behavior).
- R11: Unyielding variants error on `Await`, `Spawn`, and `Yield` nodes.
- R12: `PENDING_INTERP_CALL` struct, thread-local, stores, and retrieval are deleted.
- R13: Interpreted function shims are self-contained — they capture VM, body, env, and params.
- R14: Yielding shims spawn a `tokio::task::spawn_local` task and return `Value::Future`.
- R15: Unyielding shims execute the body synchronously and return the result.
- R16: `call_function_inner` is synchronous.
- R17: `apply` is a normal compiled function — no special-case intercept in `call_function_inner`.
- R18: `apply(yielding_fn, [args])` returns `Value::Future`.
- R19: `apply(unyielding_fn, [args])` returns the result directly.
- R20: `FunctionCall` in yielding context performs implied await if the result is `Value::Future`.
- R21: `Spawn` in yielding context returns the `Value::Future` from the shim directly.
- R22: Existing cross-boundary tests continue to pass.
- R23: New tests verify yielding functions through `apply`.

## Authority Order

1. GLOSSARY.md (new terms)
2. Specification docs
3. Architecture docs
4. Acceptance tests
5. Code (implementation)
6. Unit tests
7. Open questions
8. History
9. Index files

## TODO

### Phase 1: Documentation (authority order items 1–3)

- [x] 1. **GLOSSARY.md — add new terms** — `GLOSSARY.md`
  - Add `code analyzer`: "A module that performs static analysis on AST nodes. The stub analyzer classifies functions as yielding or unyielding at declaration time by walking the function body. See [docs/architecture/vm.md](docs/architecture/vm.md)."
  - Add `yielding classification`: "The determination of whether a function is yielding (may yield to the runtime during execution) or unyielding (executes synchronously without yielding). Performed by the code analyzer at function declaration time."
  - Update `function implementation`: mention self-contained shims that execute directly instead of using `PENDING_INTERP_CALL`.

- [x] 2. **Concurrency spec — two-state yielding model** — `docs/spec/concurrency.md`
  - Replace the three-state yielding classification table with two states: yielding (`Some(true)`) and unyielding (`Some(false)` or `None`).
  - Remove the "Ambiguous" row and its pass-through behavior from the table.
  - Remove the paragraph starting "The ambiguous case exists because the prototype does not yet categorize all functions..."
  - Add a note that `Option<bool>` is retained with `None` treated as unyielding.
  - In the "Implied Await" section, add: "When `call_function_inner` returns a `Value::Future` from a bare function call (no `await` or `spawn`), the yielding interpreter path implicitly awaits the future."

- [x] 3. **Architecture docs — self-contained shims, analyzer, exec variants** — `docs/architecture/vm.md`
  - Remove PENDING_INTERP_CALL documentation (the `InterpCall` description and thread-local mechanism).
  - Add "Code Analyzer" section describing `analyzer.rs`: stub implementation, AST walking, yielding node detection, function call lookup, known limitations (no forward references, no cycles).
  - Update "Shim-Only Function Dispatch" section: shims are self-contained. Yielding shims spawn a `spawn_local` task and return `Value::Future`. Unyielding shims call `exec_statement_unyielding` synchronously.
  - Add "Execution Variants" section: `exec_statement_yielding`/`eval_expression_yielding` (async) vs `exec_statement_unyielding`/`eval_expression_unyielding` (sync). Explain the split rationale.
  - Update `call_function_inner` documentation to reflect synchronous dispatch.

- [x] 4. **Runtime architecture doc** — `docs/architecture/runtime.md`
  - Update `IshFunction` documentation: `has_yielding_entry` remains `Option<bool>` but `None` is treated as unyielding.

**CHECKPOINT 1:** Verify documentation is internally consistent. The spec, architecture, and glossary should agree on the two-state model and self-contained shim architecture.

---

### Phase 2: Acceptance Tests (authority order item 4)

- [x] 5. **Add analyzer classification tests** — `proto/ish-tests/functions/analyzer.sh`
  - Test: `async fn` classified as yielding → `await async_fn()` succeeds.
  - Test: function containing `await` classified as yielding.
  - Test: function containing `spawn` classified as yielding.
  - Test: function with no yielding nodes classified as unyielding → `await unyielding_fn()` throws E012.
  - Test: function calling a yielding function (without explicit await) classified as yielding.
  - Test: lambda classified correctly.

- [x] 6. **Add cross-boundary yielding tests** — `proto/ish-tests/functions/cross_boundary.sh`
  - Test: `apply(async_fn, [args])` returns a Future → `await apply(async_fn, [10])` returns the result.
  - Test: `apply(unyielding_fn, [args])` returns the result directly.
  - Test: `type_of(apply(async_fn, [10]))` is `"Future"`.

- [x] 7. **Add unyielding context error tests** — `proto/ish-tests/concurrency/unyielding_context.sh`
  - Test: unyielding function body containing `await` → error at declaration (analyzer rejects).
  - Test: unyielding function body containing `spawn` → error at declaration.
  - Test: unyielding function body containing `yield` → error at declaration.

**CHECKPOINT 2:** Run `bash proto/ish-tests/run_all.sh`. Existing tests pass. New tests fail (expected — code not yet changed).

---

### Phase 3: Code — Analyzer (authority order item 5)

- [x] 8. **Create `analyzer.rs`** — `proto/ish-vm/src/analyzer.rs`
  - Define `pub enum YieldingClassification { Yielding, Unyielding }`.
  - Implement `pub fn classify_function(body: &Statement, is_async: bool, env: &Environment) -> Result<YieldingClassification, RuntimeError>`.
  - Implement `fn contains_yielding_node(stmt: &Statement, env: &Environment) -> Result<bool, RuntimeError>` — recursive AST walker.
  - Implement `fn expr_contains_yielding(expr: &Expression, env: &Environment) -> Result<bool, RuntimeError>` — expression walker.
  - Yielding nodes: `Expression::Await`, `Expression::Spawn`, `Statement::Yield`, `Expression::CommandSubstitution`.
  - For `Expression::FunctionCall { callee: Expression::Identifier(name), .. }`: look up `name` in `env`. If it resolves to `Value::Function(f)` with `f.has_yielding_entry == Some(true)`, return true. If the identifier is not defined, return `Err(RuntimeError)`.
  - Do NOT recurse into `Statement::FunctionDecl` or `Expression::Lambda` bodies.

- [x] 9. **Register analyzer module** — `proto/ish-vm/src/lib.rs`
  - Add `pub mod analyzer;`.

- [x] 10. **Wire analyzer into function declaration** — `proto/ish-vm/src/interpreter.rs`
  - In the `Statement::FunctionDecl` handler: call `crate::analyzer::classify_function(body, *is_async, env)?`. Map `Yielding` → `Some(true)`, `Unyielding` → `Some(false)`. Replace the existing `has_yielding_entry` logic that only checks `unyielding_depth`.
  - In the `Expression::Lambda` handler: call the analyzer similarly. Replace the hardcoded `has_yielding_entry: None`.

**CHECKPOINT 3:** `cd proto && cargo build --workspace` compiles. `cd proto && cargo test --workspace` passes. Functions are now classified at declaration time.

---

### Phase 4: Code — Remove Ambiguous Case

- [x] 11. **Update await handling** — `proto/ish-vm/src/interpreter.rs`
  - In `Expression::Await`: change the E012 check from `f.has_yielding_entry == Some(false)` to `f.has_yielding_entry != Some(true)`. This makes `None` also throw E012.
  - Remove the fall-through `Ok(val)` case for non-Future results at the end of the Await handler (the "Non-future result from ambiguous function — pass through" comment). Replace with an error: if the result is not `Value::Future`, throw E012 ("expected Future from yielding function").

- [x] 12. **Update spawn handling** — `proto/ish-vm/src/interpreter.rs`
  - In `Expression::Spawn`: change the E013 check from `f.has_yielding_entry == Some(false)` to `f.has_yielding_entry != Some(true)`.

**CHECKPOINT 4:** `cd proto && cargo test --workspace` passes. No ambiguous pass-through. Analyzer classification tests pass.

---

### Phase 5: Code — Extract Helpers and Create Execution Variants

- [x] 13. **Extract pure computation helpers** — `proto/ish-vm/src/interpreter.rs`
  - Extract `apply_binary_op(op, left, right) -> Result<Value, RuntimeError>` from the `BinaryOp` match arm.
  - Extract `apply_unary_op(op, operand) -> Result<Value, RuntimeError>` from the `UnaryOp` match arm.
  - Extract `define_variable(env, name, mutable, value, type_annotation) -> Result<ControlFlow, RuntimeError>` from `VariableDecl`.
  - Extract `perform_assignment(vm, env, target, value) -> Result<ControlFlow, RuntimeError>` from `Assignment` (for the final write step, after evaluating sub-expressions — note: the target may be `PropertyAccess` or `IndexAccess` which requires evaluating the object, so the helper takes the already-evaluated value and handles the write).
  - Extract `build_object(pairs: Vec<(String, Value)>) -> Value` from `ObjectLiteral`.
  - Extract `build_list(items: Vec<Value>) -> Value` from `ListLiteral`.
  - Extract `controlflow_from_result(result: Result<ControlFlow, RuntimeError>) -> Result<Value, RuntimeError>` — converts `ControlFlow::Return(v) | ExprValue(v)` → `Ok(v)`, `None` → `Ok(Null)`, `Throw(v)` → `Err`.
  - Verify all helpers are used in the existing `exec_statement`/`eval_expression` before creating variants.

- [x] 14. **Create `exec_statement_unyielding`** — `proto/ish-vm/src/interpreter.rs`
  - Signature: `fn exec_statement_unyielding(vm: &Rc<RefCell<IshVm>>, task: &mut TaskContext, stmt: &Statement, env: &Environment) -> Result<ControlFlow, RuntimeError>`.
  - No `YieldContext` parameter. No `async`. No `Box::pin`.
  - Copy the match structure from `exec_statement`, replacing:
    - All `Self::eval_expression(...)​.await?` → `Self::eval_expression_unyielding(...)?`
    - All `Self::exec_statement(...)​.await?` → `Self::exec_statement_unyielding(...)?`
    - Remove `yc.check_yield_budget().await` calls.
    - `Statement::Yield` → return error (yield in unyielding context).
    - `Statement::Annotated` with `@[unyielding]` → no-op (already unyielding), just recurse.
  - Use extracted helpers where applicable.

- [x] 15. **Create `eval_expression_unyielding`** — `proto/ish-vm/src/interpreter.rs`
  - Signature: `fn eval_expression_unyielding(vm: &Rc<RefCell<IshVm>>, task: &mut TaskContext, expr: &Expression, env: &Environment) -> Result<Value, RuntimeError>`.
  - No `YieldContext` parameter. No `async`. No `Box::pin`.
  - Copy the match structure from `eval_expression`, replacing:
    - All `Self::eval_expression(...)​.await?` → `Self::eval_expression_unyielding(...)?`
    - All `Self::exec_statement(...)​.await?` → `Self::exec_statement_unyielding(...)?`
    - `Expression::Await` → return error (await in unyielding context).
    - `Expression::Spawn` → return error (spawn in unyielding context).
    - `Expression::FunctionCall` → `Self::call_function_inner(vm, &func_val, &arg_vals)?` (sync, return result as-is).
    - `Expression::CommandSubstitution` → return error (shell commands are yielding).
  - Use extracted helpers where applicable.

- [x] 16. **Rename existing functions to yielding variants** — `proto/ish-vm/src/interpreter.rs`
  - Rename `exec_statement` → `exec_statement_yielding`. Update all internal callers.
  - Rename `eval_expression` → `eval_expression_yielding`. Update all internal callers.
  - Update visibility: both yielding and unyielding variants are `pub(crate)`.
  - Update `run()`, `pop_and_run_defers()`, and any other callers to use `_yielding` suffix.

**CHECKPOINT 5:** `cd proto && cargo build --workspace` compiles. `cd proto && cargo test --workspace` passes. Both yielding and unyielding paths exist.

---

### Phase 6: Code — Self-Contained Shims and Synchronous call_function_inner

- [x] 17. **Update function declaration shims** — `proto/ish-vm/src/interpreter.rs`
  - In `Statement::FunctionDecl`: based on analyzer classification, create either:
    - **Unyielding shim**: captures `vm.clone()`, `body.clone()`, `env.clone()`, `param_names.clone()`. Shim creates child env, defines params, creates `TaskContext`, calls `Self::exec_statement_unyielding(...)`, runs defers (synchronous), converts `ControlFlow` to `Value`.
    - **Yielding shim**: captures same. Shim creates child env, defines params, clones captures into async block, calls `tokio::task::spawn_local(async move { ... exec_statement_yielding ... })`, wraps `JoinHandle` in `FutureRef::new()`, returns `Ok(Value::Future(...))`.
  - Remove the old shim that stores `PENDING_INTERP_CALL`.

- [x] 18. **Update lambda shims** — `proto/ish-vm/src/interpreter.rs`
  - In `Expression::Lambda`: same pattern as function declaration — analyzer classifies, create appropriate shim variant.
  - Remove the old shim that stores `PENDING_INTERP_CALL`.

- [x] 19. **Delete PENDING_INTERP_CALL** — `proto/ish-vm/src/interpreter.rs`
  - Delete the `InterpCall` struct.
  - Delete the `PENDING_INTERP_CALL` thread-local.
  - Delete any remaining references.

- [x] 20. **Make `call_function_inner` synchronous** — `proto/ish-vm/src/interpreter.rs`
  - Change signature from `fn call_function_inner<'a>(...) -> Pin<Box<dyn Future<...> + 'a>>` to `fn call_function_inner(vm: &Rc<RefCell<IshVm>>, func: &Value, args: &[Value]) -> Result<Value, RuntimeError>`.
  - Remove `task` and `yc` parameters (shims handle their own).
  - Remove `Box::pin(async move { ... })` wrapper.
  - Remove the PENDING_INTERP_CALL check (`let pending = ...` block).
  - Keep: arity check, parameter type audit, shim call `(f.shim)(args)?`, return type audit.
  - Keep: ledger builtin intercepts (`active_standard`, `feature_state`, etc.) — these need VM access and are already synchronous. Remove only the `apply` intercept.
  - Remove the `builtin_apply` method entirely.

- [x] 21. **Update FunctionCall in yielding context** — `proto/ish-vm/src/interpreter.rs`
  - In `eval_expression_yielding`, `Expression::FunctionCall` arm:
    - Call `Self::call_function_inner(vm, &func_val, &arg_vals)?` (no `.await`).
    - Add implied await: if result is `Value::Future`, take the handle and `.await` it. If not, return as-is.
  - This replaces the current `Self::call_function_inner(...).await`.

- [x] 22. **Update Await handler in yielding context** — `proto/ish-vm/src/interpreter.rs`
  - In `eval_expression_yielding`, `Expression::Await` arm:
    - Call `Self::call_function_inner(vm, &callee_val, &arg_vals)?` (no `.await`).
    - Await the result: if `Value::Future`, take handle and `.await`. If not, error E012.

- [x] 23. **Update Spawn handler in yielding context** — `proto/ish-vm/src/interpreter.rs`
  - In `eval_expression_yielding`, `Expression::Spawn` arm:
    - Evaluate callee and args as before.
    - Call `Self::call_function_inner(vm, &callee_val, &arg_vals)?` (no `.await`).
    - The result should be `Value::Future` (from the yielding shim). Return it directly.
    - Remove the `spawn_local { call_function_inner(...).await }` wrapping.
    - Known limitation: VM clone for ledger isolation is lost. Note as TODO for future work.

- [x] 24. **Update `apply` registration** — `proto/ish-vm/src/builtins.rs`
  - Replace the current `register_apply` function. The new shim:
    - Validates 2 arguments (function and list).
    - Extracts the function value and argument list.
    - Calls `(f.shim)(&arg_list)` directly.
    - Returns the result (may be `Value::Future` or direct value).
  - Set `has_yielding_entry` to `Some(false)` — `apply` itself is unyielding.

**CHECKPOINT 6:** `cd proto && cargo build --workspace` compiles. `cd proto && cargo test --workspace` passes. `bash proto/ish-tests/run_all.sh` — all acceptance tests pass including new ones.

---

### Phase 7: Unit Tests (authority order item 6)

- [x] 25. **Add analyzer unit tests** — `proto/ish-vm/src/analyzer.rs` (or `proto/ish-vm/tests/`)
  - Test classify_function with async fn → Yielding.
  - Test classify_function with function containing Await → Yielding.
  - Test classify_function with function containing Spawn → Yielding.
  - Test classify_function with function containing Yield → Yielding.
  - Test classify_function with function containing CommandSubstitution → Yielding.
  - Test classify_function with plain function → Unyielding.
  - Test classify_function with nested function declaration → does not recurse (parent stays unyielding even if inner function has await).
  - Test classify_function with call to yielding function → Yielding.
  - Test classify_function with call to undefined function → Error.

**CHECKPOINT 7:** `cd proto && cargo test --workspace` — all unit tests pass.

---

### Phase 8: Cleanup and Final Documentation

- [x] 26. **Close open questions** — `docs/project/open-questions.md`
  - Mark "Function yielding categorization at declaration" as done with a reference to the stubbed analyzer implementation.
  - Mark "Code analyzer scope" as partially done — stub analyzer classifies yielding; more analysis passes to come.

- [x] 27. **History** — `docs/project/history/2026-04-03-stubbed-analyzer/summary.md`
  - Append implementation narrative to the existing summary.

- [x] 28. **Index files** — `docs/project/history/INDEX.md`
  - Verify the existing entry is correct (already added during proposal phase).

**CHECKPOINT 8 (Final):** Run full verification:
  - `cd proto && cargo build --workspace` — compiles clean.
  - `cd proto && cargo test --workspace` — all tests pass.
  - `bash proto/ish-tests/run_all.sh` — all acceptance tests pass.
  - Review: no references to `PENDING_INTERP_CALL` remain in codebase.
  - Review: `apply` has no special-case intercept in `call_function_inner`.
  - Review: all functions declared at runtime have `has_yielding_entry` set to `Some(true)` or `Some(false)`.

---

## Reference

### Current PENDING_INTERP_CALL Locations

- **Definition**: `proto/ish-vm/src/interpreter.rs` lines 13–26 — `InterpCall` struct and `PENDING_INTERP_CALL` thread-local.
- **Store in FunctionDecl**: `proto/ish-vm/src/interpreter.rs` ~line 487 — shim closure stores `InterpCall`.
- **Store in Lambda**: `proto/ish-vm/src/interpreter.rs` ~line 975 — shim closure stores `InterpCall`.
- **Retrieval in call_function_inner**: `proto/ish-vm/src/interpreter.rs` ~line 1432.
- **apply intercept**: `proto/ish-vm/src/interpreter.rs` ~line 1425 — `"apply" if f.has_yielding_entry.is_some()`.
- **builtin_apply method**: `proto/ish-vm/src/interpreter.rs` ~line 1518.

### Ledger Builtin Intercepts (Kept)

The following intercepts remain in `call_function_inner` because the ledger builtins need VM access and are registered before the VM is wrapped in `Rc<RefCell<IshVm>>`:
- `active_standard`, `feature_state`, `has_standard`, `has_entry_type`, `ledger_state`, `has_entry`

Their stub registrations in `builtins.rs` (lines 43–63) also remain unchanged. Removing these intercepts requires changing the VM construction order so builtins can capture a VM reference — this is out of scope for this plan.

### Current has_yielding_entry Usage

- `IshFunction` struct: `proto/ish-runtime/src/value.rs` ~line 92
- `new_compiled_function`: `proto/ish-runtime/src/value.rs` ~line 111
- All builtins use `Some(false)`: `proto/ish-vm/src/builtins.rs`
- Ledger stubs use `Some(false)`: `proto/ish-vm/src/builtins.rs` ~line 60
- `apply` stub uses `Some(false)`: `proto/ish-vm/src/builtins.rs` ~line 78
- FunctionDecl handler: `proto/ish-vm/src/interpreter.rs` ~line 471 — currently checks `unyielding_depth`
- Lambda handler: `proto/ish-vm/src/interpreter.rs` ~line 969 — hardcoded `None`
- Await check: `proto/ish-vm/src/interpreter.rs` ~line 1065 — checks `Some(false)`
- Spawn check: `proto/ish-vm/src/interpreter.rs` ~line 1123 — checks `Some(false)`
- apply intercept: `proto/ish-vm/src/interpreter.rs` ~line 1425 — checks `is_some()`

### YieldContext and TaskContext

- `YieldContext`: `proto/ish-vm/src/interpreter.rs` ~line 75 — `unyielding_depth`, `budget_start`, `budget_duration`.
- `TaskContext`: `proto/ish-vm/src/interpreter.rs` ~line 38 — `defer_stack`, `async_stack`.
- Unyielding shims create fresh `TaskContext` (for defer management). No `YieldContext`.
- Yielding shims create both `TaskContext` and `YieldContext` inside the spawned task.

### Known Limitations After Implementation

1. **No forward references**: Functions must be declared before they are called. The analyzer errors on calls to undefined functions.
2. **No call cycles**: Mutually recursive functions are not supported by the stub analyzer.
3. **No ledger isolation for spawn**: The `Spawn` handler no longer clones the VM for spawned tasks. Spawned tasks share the parent's ledger state.
4. **Indirect call classification**: The analyzer only checks direct `Identifier` calls. Indirect calls (through variables, higher-order functions) are not analyzed — the function may be misclassified.

## Referenced by

- [docs/project/proposals/stubbed-analyzer.md](../proposals/stubbed-analyzer.md)
