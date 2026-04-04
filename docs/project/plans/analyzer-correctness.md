---
title: "Plan: Analyzer Correctness Fixes"
category: plan
audience: [ai-dev]
status: completed
last-verified: 2026-04-04
depends-on:
  - docs/project/proposals/analyzer-correctness.md
  - docs/spec/concurrency.md
  - docs/architecture/vm.md
  - docs/errors/INDEX.md
  - proto/ish-vm/src/analyzer.rs
  - proto/ish-vm/src/interpreter.rs
  - proto/ish-ast/src/lib.rs
  - proto/ish-parser/src/ast_builder.rs
---

# Plan: Analyzer Correctness Fixes

*Derived from [analyzer-correctness.md](../proposals/analyzer-correctness.md) on 2026-04-04.*

## Overview

Fix nine correctness issues in the analyzer and interpreter introduced during the stubbed-analyzer implementation: add an `is_yielding` builtin for test introspection; reclassify `spawn` as non-yielding and allow it in unyielding execution contexts; make undefined function calls an error instead of silently treating them as unyielding; restrict implied await to calls of yielding functions (fixing `apply` behavior); expand the `await` grammar to accept any expression (restructuring the `Await` AST node and adding E014); and repair several acceptance test files that have incorrect or missing tests.

## Requirements

- R1: `is_yielding(f)` is a builtin that returns `true` if `f` has `has_yielding_entry == Some(true)`, `false` otherwise.
- R2: `is_yielding` called on a non-function value throws E004.
- R3: `spawn expr` does not make the enclosing function yielding in the analyzer.
- R4: `spawn` is valid in an unyielding execution context — `eval_expression_unyielding` does not error on `Expression::Spawn`.
- R5: Calls to undefined functions in a function body return an analyzer error at declaration time (not silent unyielding).
- R6: Implied await in the `FunctionCall` yielding path is guarded by the callee's `has_yielding_entry == Some(true)`. Unyielding callees that return a `Value::Future` are not awaited.
- R7: `apply(async_fn, args)` returns `Value::Future`; the caller must explicitly await it.
- R8: `await` accepts any expression in the grammar (not just a function call).
- R9: The `Await` AST node has the shape `{ expr: Box<Expression> }`, not `{ callee, args }`.
- R10: `await non_future_value` throws E014 (`AwaitNonFuture`) at runtime.
- R11: `await func()` continues to work; the `FunctionCall` expression is the expr in the new AST node.
- R12: E012 check (await applied to unyielding function) is restricted to the case where the `await`-ed expression is a `FunctionCall` to an identifiably unyielding callee.
- R13: Any `await` node still classifies the enclosing function as yielding in the analyzer.
- R14: `spawn` still requires a function call as its operand (grammar unchanged for `spawn`).
- R15: Unyielding context tests in `unyielding_context.sh` define `some_fn` before use and remove the invalid spawn-errors test.
- R16: The shell integration test removes the redundant `async` keyword from a function classified as yielding by command substitution alone.
- R17: Acceptance tests cover `is_yielding` for `async fn`, unyielding `fn`, `fn` with spawn, `fn` with command substitution, `fn` with shell command, `fn` calling undefined function (error).
- R18: `spawn_await.sh` contains tests for `await variable`, `await complex_expression`, and `await non_future` (E014).
- R19: `cross_boundary_yielding.sh` is fully rewritten to reflect correct behavior: `apply(async_fn, args)` returns a future; `apply(unyielding_fn, args)` returns the result; `is_yielding(apply)` is `false`.

## Authority Order

1. GLOSSARY.md
2. Roadmap (in progress)
3. Specification docs (`docs/spec/errors.md`, `docs/spec/concurrency.md`)
4. Architecture docs (`docs/architecture/vm.md`)
5. Error catalog (`docs/errors/INDEX.md`)
6. Acceptance tests
7. Code (implementation)
8. Unit tests (inline in `analyzer.rs`)
9. Roadmap (completed)
10. History
11. Index files

## TODO

### Phase 1: Glossary and Roadmap

- [x] 1. **GLOSSARY.md — update "implied await" definition** — `GLOSSARY.md`
  - Change the current definition to clarify that implied await applies only when calling a **yielding** function. Unyielding functions that happen to return a `Value::Future` do not trigger implied await.
  - Current text: "the interpreter automatically awaits a `Future` returned by a bare function call"
  - New text should read: "the interpreter automatically awaits a `Future` returned by a call to a **yielding** function. Calling an unyielding function that returns a `Value::Future` does not trigger implied await — the future is returned as-is."

- [x] 2. **Roadmap — add analyzer-correctness entry as in-progress** — `docs/project/roadmap.md`
  - Add under "In Progress": `- [ ] Analyzer correctness fixes (is_yielding builtin, spawn reclassification, implied await scope, await grammar expansion)`

### Phase 2: Specification Docs

- [x] 3. **docs/spec/errors.md — add E014** — `docs/spec/errors.md`
  - Add row to the error code table:
    `| E014 | TypeError | Await type mismatch — \`await\` applied to a non-future value |`
  - Insert after the E013 row.

- [x] 4. **docs/spec/concurrency.md — update Grammar Restrictions section** — `docs/spec/concurrency.md`
  - Change the `await_op` line from `await_op ~ call_expr` to `await_op ~ expr`.
  - Remove the sentence "`await` and `spawn` syntactically require a function call as their operand" and replace with: "`await` accepts any expression as its operand. `spawn` still requires a function call."
  - Add: "If the `await`-ed expression does not evaluate to a `Value::Future`, ish throws E014 (`AwaitNonFuture`)."
  - Update the description of the `Await` AST node: it now has the shape `{ expr: Box<Expression> }` rather than `{ callee, args }`.

- [x] 5. **docs/spec/concurrency.md — update Implied Await section** — `docs/spec/concurrency.md`
  - Add the following rule: implied await is guarded by the callee's yielding classification. Only calls to functions with `has_yielding_entry == Some(true)` trigger implied await. Unyielding functions that return a `Value::Future` (e.g. `apply`) do not trigger implied await.
  - Add an example contrasting `await async_fn()` (implied await applies) vs. `apply(async_fn, args)` (returns `Value::Future` as-is).
  - Update: "The only way to obtain a `Future` value (rather than the resolved value) is via `spawn`." → Also via unyielding functions that return a future (e.g. `apply`).

### Phase 3: Architecture Docs and Error Catalog

- [x] 6. **docs/architecture/vm.md — update apply and FunctionCall descriptions** — `docs/architecture/vm.md`
  - In the `apply` description: note that `apply` is unyielding, so calling `apply(async_fn, args)` returns `Value::Future` without implied await. The caller must explicitly `await` the result.
  - In the FunctionCall yielding-path description: update to show that implied await is conditioned on `func.has_yielding_entry == Some(true)`.
  - In the unyielding execution path description: note that `Spawn` is valid in unyielding context — it is not an error.

- [x] 7. **docs/errors/INDEX.md — add E014** — `docs/errors/INDEX.md`
  - Add entry after E013:
    `| E014 | AwaitNonFuture | TypeError | Await type mismatch — \`await\` applied to a non-future value | interpreter.rs (Expression::Await, non-Future value) |`

### Phase 4: Acceptance Tests

- [x] 8. **proto/ish-tests/functions/analyzer.sh — update and extend** — `proto/ish-tests/functions/analyzer.sh`

  **8a.** Replace the "fn with spawn classified yielding" test (which tests incorrect behavior) with "fn with spawn classified unyielding":
  ```bash
  output=$(run_ish 'async fn inner() { return 5 }
                    fn wrapper() { return spawn inner() }
                    println(is_yielding(wrapper))')
  assert_output "fn with spawn classified unyielding" "false" "$output"
  ```

  **8b.** Update any existing classification tests that use `type_of(await ...)` patterns to use `is_yielding(...)` instead (the `is_yielding` builtin is now available).

  **8c.** Add test for undefined function call erroring at declaration time:
  ```bash
  output=$(run_ish 'fn bad() { undefined_fn() }')
  assert_output_contains "undefined fn call errors at declaration" "E005" "$output"
  ```

  **8d.** Add test for command substitution classified as yielding:
  ```bash
  output=$(run_ish 'fn uses_subst() { let r = $(echo hello); return r }
                    println(is_yielding(uses_subst))')
  assert_output "fn with command substitution is yielding" "true" "$output"
  ```

  **8e.** Add test for shell command classified as yielding:
  ```bash
  output=$(run_ish 'fn runs_shell() { echo hello }
                    println(is_yielding(runs_shell))')
  assert_output "fn with shell command is yielding" "true" "$output"
  ```

- [x] 9. **proto/ish-tests/concurrency/unyielding_context.sh — rewrite** — `proto/ish-tests/concurrency/unyielding_context.sh`

  Replace all three existing tests with:
  ```bash
  # @[unyielding] function body containing await → analyzer error at declaration
  output=$(run_ish 'async fn some_fn() { return 1 }
                    @[unyielding] fn bad() { await some_fn() }')
  assert_output_contains "unyielding fn with await errors" "E" "$output"

  # @[unyielding] function body containing yield → analyzer error at declaration
  output=$(run_ish '@[unyielding] fn bad() { yield }')
  assert_output_contains "unyielding fn with yield errors" "E" "$output"

  # @[unyielding] function body containing spawn → no error (spawn is allowed)
  output=$(run_ish 'async fn some_fn() { return 1 }
                    @[unyielding] fn ok() { spawn some_fn() }
                    println("ok")')
  assert_output "unyielding fn with spawn is valid" "ok" "$output"
  ```

- [x] 10. **proto/ish-tests/concurrency/shell_integration.sh — remove redundant `async`** — `proto/ish-tests/concurrency/shell_integration.sh`
  - Find the "shell command in awaited task" test.
  - Change `async fn work()` to `fn work()`. The function body uses `$(...)`, so the analyzer classifies it as yielding without the keyword.

- [x] 11. **proto/ish-tests/concurrency/spawn_await.sh — add `await expr` tests** — `proto/ish-tests/concurrency/spawn_await.sh`

  Add three new tests:
  ```bash
  # await a variable holding a future
  output=$(run_ish 'async fn work() { return 42 }
                    let f = spawn work()
                    println(await f)')
  assert_output "await variable resolves future" "42" "$output"

  # await a complex expression (list index)
  output=$(run_ish 'async fn work(x) { return x + 1 }
                    let fs = [spawn work(1), spawn work(2)]
                    println(await fs[0])')
  assert_output "await complex expression resolves future" "2" "$output"

  # await a non-future value → E014
  output=$(run_ish 'let x = 42
                    await x')
  assert_output_contains "await non-future throws E014" "E014" "$output"
  ```

- [x] 12. **proto/ish-tests/functions/cross_boundary_yielding.sh — full rewrite** — `proto/ish-tests/functions/cross_boundary_yielding.sh`

  Replace all existing content with:
  ```bash
  # apply(async_fn, args) returns a Future — no implied await (apply is unyielding)
  output=$(run_ish 'async fn work(x) { return x + 1 }
                    println(type_of(apply(work, [10])))')
  assert_output "apply async fn returns future" "future" "$output"

  # The returned future can be awaited
  output=$(run_ish 'async fn work(x) { return x + 1 }
                    let f = apply(work, [10])
                    println(await f)')
  assert_output "apply async fn future resolves correctly" "11" "$output"

  # apply(unyielding_fn, args) returns the result directly
  output=$(run_ish 'fn add(a, b) { return a + b }
                    println(apply(add, [3, 4]))')
  assert_output "apply unyielding fn direct result" "7" "$output"

  # apply is itself unyielding
  output=$(run_ish 'println(is_yielding(apply))')
  assert_output "apply is unyielding" "false" "$output"
  ```

- [x] 13. **Checkpoint:** All acceptance tests written. Verify each test file is syntactically valid shell.

### Phase 5: Code — Feature 1 (`is_yielding` builtin)

- [x] 14. **proto/ish-vm/src/builtins.rs — add `is_yielding` builtin**
  - Add a new builtin function `is_yielding`.
  - Signature: `fn(Value) -> Result<Value, RuntimeError>`.
  - Implementation:
    - Match `Value::Function(f)` → return `Value::Bool(f.has_yielding_entry == Some(true))`.
    - All other values → return `RuntimeError::type_mismatch(...)` (E004).
  - Register `is_yielding` in the builtin dispatch table (wherever `println`, `type_of`, `apply`, etc. are registered).

### Phase 6: Code — Feature 2 (spawn reclassification) and Feature 3 (undefined fn error)

- [x] 15. **proto/ish-vm/src/analyzer.rs — fix spawn classification**
  - In `expr_contains_yielding`, locate the `Expression::Spawn { .. } => Ok(true)` arm.
  - Replace it with an arm that recurses into spawn's arguments (does not return `Ok(true)` for spawn itself). The spawn arguments may contain yielding sub-expressions; recurse into them and return their result.
  - The spawn target (the function being spawned) is not itself a yielding classification source — the caller does not yield by virtue of calling `spawn`.

- [x] 16. **proto/ish-vm/src/analyzer.rs — fix undefined function call behavior**
  - In `expr_contains_yielding`, locate the `FunctionCall` branch's `Err(_)` arm (currently: silently treats as unyielding with a comment about forward references).
  - Replace with:
    ```rust
    Err(_) => {
        return Err(RuntimeError::system_error(
            ErrorCode::UndefinedVariable,
            &format!("undefined function '{}' — forward references are not supported", name),
        ));
    }
    ```

- [x] 17. **proto/ish-vm/src/interpreter.rs — allow Spawn in unyielding path**
  - In `eval_expression_unyielding`, locate the error arm for `Expression::Spawn`.
  - Remove the error. Instead, handle `Spawn` in the unyielding path: call `call_function_inner` on the spawn target and return `Value::Future` directly (do not await). The unyielding caller does not suspend.

### Phase 7: Code — Feature 4 (implied await scope)

- [x] 18. **proto/ish-vm/src/interpreter.rs — guard implied await by callee yielding classification**
  - In `eval_expression_yielding`, locate the `FunctionCall` handler where implied await occurs (the code that checks if `call_function_inner` returned a `Value::Future` and awaits it).
  - Add a guard: only perform implied await if `func.has_yielding_entry == Some(true)`.
  - If the callee is unyielding (or `None`), return the result as-is even if it is a `Value::Future`.
  - Target code structure:
    ```rust
    let result = call_function_inner(...)?;
    if func.has_yielding_entry == Some(true) {
        if let Value::Future(f) = result {
            return Ok(f.await?);
        }
    }
    Ok(result)
    ```

### Phase 8: Code — Feature 5 (`await expr` grammar expansion)

Feature 5 is a breaking AST change. All four files must be updated together to keep the project compilable.

- [x] 19. **proto/ish-ast/src/lib.rs — restructure `Await` AST node**
  - Find the `Expression::Await` variant. It currently has fields `callee: Box<Expression>` and `args: Vec<Expression>`.
  - Change it to: `Expression::Await { expr: Box<Expression> }`.
  - This will cause compile errors in all consumers — fix them in the steps below.

- [x] 20. **proto/ish-parser/src/ast_builder.rs — update Await construction**
  - Find where the parser builds `Expression::Await` from the parse tree.
  - Currently it extracts a callee and arguments (mirroring `FunctionCall`). Change it to extract a single expression — the entire operand after the `await` keyword.
  - The grammar production for `await` (in the pest grammar, likely `proto/ish-parser/src/lib.rs` or an embedded `.pest` file) must be updated: change `await_op ~ call_expr` to `await_op ~ expr`. Locate the grammar file and make this change.
  - After the grammar change, the parsed operand is a full expression. Pass it directly as `expr` in the new `Await` variant.
  - The E012 check (await on unyielding callee): if the parsed expression is a `FunctionCall`, check whether the callee is identifiably unyielding and emit E012 at parse/analysis time as before. If the expression is not a `FunctionCall`, skip the E012 check (runtime E014 is the fallback).

- [x] 21. **proto/ish-vm/src/analyzer.rs — update Await handling**
  - Find the `Expression::Await { callee, args }` match arm in `expr_contains_yielding`.
  - Update the pattern to `Expression::Await { expr }`.
  - The arm must still return `Ok(true)` — any `await` node makes the enclosing function yielding. Recurse into `expr` to detect nested yielding nodes as well.
  - Check `proto/ish-stdlib/src/analyzer.rs` for any `Expression::Await` handling and update it identically.

- [x] 22. **proto/ish-vm/src/interpreter.rs — update `Await` handler**
  - Find `Expression::Await { callee, args }` in `eval_expression_yielding`.
  - Replace with `Expression::Await { expr }` and implement the new runtime behavior:
    ```rust
    Expression::Await { expr } => {
        let val = eval_expression_yielding(expr, ...)?;
        match val {
            Value::Future(f) => Ok(f.await?),
            Value::Function(func) if func.has_yielding_entry == Some(false) => {
                Err(RuntimeError::coded(ErrorCode::AwaitUnyielding, ...))
            }
            _ => Err(RuntimeError::coded(ErrorCode::AwaitNonFuture,
                "await requires a Future value")),
        }
    }
    ```
  - Remove any previous `eval_expression_yielding(callee)` + argument-evaluation logic from this arm.
  - Ensure `ErrorCode::AwaitNonFuture` exists in the `ErrorCode` enum (add it if not present, mapped to E014).

- [x] 23. **proto/ish-runtime/src/error.rs (or wherever ErrorCode lives) — add AwaitNonFuture**
  - Add `AwaitNonFuture` to the `ErrorCode` enum with numeric value `14`.
  - Ensure it maps to `E014` in any display/formatting logic.

- [x] 24. **Checkpoint:** Project compiles. Run `cargo check` (or `cargo build`) across all crates. Fix any remaining `Expression::Await` pattern matches that were missed.

### Phase 9: Unit Tests

- [x] 25. **proto/ish-vm/src/analyzer.rs — update and add unit tests**

  **25a.** Find `fn_with_spawn_is_yielding` (around line 355). Update it to assert `Unyielding` instead of `Yielding` — a function whose body contains only `spawn` is now unyielding.

  **25b.** Add `spawn_in_unyielding_fn_is_valid` — classifying a function that contains `spawn` should succeed (no error), and the result should be `Unyielding`.

  **25c.** Find `call_to_undefined_fn_is_unyielding` (around line 445). Rename to `call_to_undefined_fn_is_error`. Update the assertion to expect `Err(...)` from `classify_function` rather than `Ok(Unyielding)`.

### Phase 10: Roadmap, History, and Index Files

- [x] 26. **Roadmap — mark analyzer-correctness completed** — `docs/project/roadmap.md`
  - Move "Analyzer correctness fixes" from "In Progress" to "Completed".
  - Also check whether "Stubbed code analyzer and yielding/unyielding function refactoring" should be marked completed (it may have been completed by the previous plan).

- [x] 27. **docs/project/proposals/INDEX.md — update status to "accepted"** — `docs/project/proposals/INDEX.md`
  - Find the `analyzer-correctness.md` row and change status from `proposal` to `accepted`.

- [x] 28. **docs/project/plans/INDEX.md — add entry** — `docs/project/plans/INDEX.md`
  - Add row: `| 2026-04-04 | Analyzer Correctness Fixes | completed | [analyzer-correctness.md](analyzer-correctness.md) |`

---

## Reference

**Decisions (all resolved):**

| # | Decision | Outcome |
|---|----------|---------|
| 1 | `is_yielding` — builtin or language feature? | builtin |
| 2 | Spawn classification test — replace or add alongside? | replace |
| 3 | `await` grammar scope | any expression; tests must cover complex expressions |
| 4 | Error code for `await non-future` | E014 |
| 5 | `Await` AST node restructure | yes (`{callee, args}` → `{expr}`) |
| 6 | Unyielding path allows `Spawn`? | yes |

**Key existing test being replaced (Feature 2):**
- File: `proto/ish-tests/functions/analyzer.sh`, lines ~23-26
- Test name: "fn with spawn classified yielding"
- Asserts `await wrapper()` works (because wrapper wrongly classified as yielding)
- This test asserts incorrect behavior and must be removed, not kept.

**E012 check location:**
- E012 currently fires when the `Await` node's callee is an identifiably unyielding function.
- After Feature 5, E012 fires only when the `await`-ed expression is a `FunctionCall` to an identifiably unyielding callee. `await variable` bypasses the check; E014 is the runtime fallback.

**Spawn recursion note (Feature 2):**
- When removing `Expression::Spawn { .. } => Ok(true)`, do not simply delete the arm.
- Replace it with an arm that recurses into the spawn's inner call expression (arguments) for nested yielding sub-expressions. The spawn call itself does not yield, but an argument expression might.

**`proto/ish-stdlib/src/analyzer.rs`:**
- This file exists and may contain its own `Expression::Await` match. Check it when updating the analyzer in steps 21 and 24.

**Grammar file location:**
- The pest grammar is likely in `proto/ish-parser/src/lib.rs` or an embedded/included `.pest` file in that crate. Locate the `await_op` production before beginning step 20.
