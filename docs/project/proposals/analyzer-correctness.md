---
title: "Proposal: Analyzer Correctness Fixes"
category: proposal
audience: [ai-dev]
status: accepted
date: 2026-04-04
depends-on:
  - docs/project/rfp/analyzer-correctness.md
  - docs/spec/concurrency.md
  - docs/architecture/vm.md
  - docs/errors/INDEX.md
  - proto/ish-vm/src/analyzer.rs
  - proto/ish-tests/functions/analyzer.sh
  - proto/ish-tests/concurrency/unyielding_context.sh
  - proto/ish-tests/concurrency/shell_integration.sh
  - proto/ish-tests/functions/cross_boundary_yielding.sh
---

# Proposal: Analyzer Correctness Fixes

*Generated from [analyzer-correctness.md](../rfp/analyzer-correctness.md) on 2026-04-04.*

---

## Decision Register

| # | Decision | Outcome |
|---|----------|---------|
| 1 | Should `is_yielding` be a builtin or a language feature? | **builtin** |
| 2 | Should the "function containing spawn is unyielding" test replace the existing test or be added alongside it? | **replace** |
| 3 | Should `await` be expanded to accept any expression, or only identifiers/variables? | **any expression; tests must include complex expressions resolving to futures** |
| 4 | What error code should `await non-future` use? | **E014** |
| 5 | Should the `Await` AST node be restructured from `{callee, args}` to `{expr}`? | **yes** |
| 6 | Should the unyielding execution path allow `Spawn` nodes? | **yes** |

---

## Feature 1: `is_yielding` Builtin

### Issues to Watch Out For

- The builtin must inspect `has_yielding_entry` on the function value, not re-run the
  analyzer. Classification is fixed at declaration time.
- Calling `is_yielding` on a non-function value should throw E004 (type mismatch), not panic.
- The builtin is a testing tool; it must be available at all assurance levels.

### Implementation

Add `is_yielding` to `builtins.rs`. Signature: `fn(Value) -> Result<Value, RuntimeError>`.

- Extract `Value::Function(f)`, return `Value::Bool(f.has_yielding_entry == Some(true))`.
- For non-function arguments, return `RuntimeError::type_mismatch(...)`.
- Update analyzer acceptance tests to use `is_yielding` for direct classification checks.

---

## Feature 2: Fix Spawn Yielding Classification

The analyzer at `proto/ish-vm/src/analyzer.rs:189` contains:

```rust
Expression::Spawn { .. } => Ok(true),
```

This is incorrect. `spawn` starts a task and returns a `Value::Future` without suspending
the caller. The caller does not yield — it continues executing. Therefore `spawn` does not
make the enclosing function yielding.

### Issues to Watch Out For

- The unyielding execution path (`exec_statement_unyielding`, `eval_expression_unyielding`)
  currently errors on `Spawn` nodes ("unyielding variants enforce runtime invariants:
  encountering Await, Spawn, or Yield nodes returns an error"). This must change: `Spawn`
  is valid in an unyielding execution context.
- The `@[unyielding]` annotation validation uses `classify_function`. Removing spawn from
  yielding nodes means `@[unyielding] fn bad() { spawn work() }` will no longer produce
  a classification error — which is the **correct** behavior.
- The unit test `fn_with_spawn_is_yielding` (analyzer.rs:355) reflects the incorrect
  behavior. It must be updated to assert `Unyielding`.
- E013 ("spawn applied to unyielding function") is not affected: E013 concerns spawning
  an unyielding *callee*, not the spawning function's own classification.

### Implementation

**analyzer.rs:**
- Remove `Expression::Spawn { .. } => Ok(true)` from `expr_contains_yielding`.
- Spawn expressions must still recurse into arguments (they may contain yielding sub-expressions).
- Update `fn_with_spawn_is_yielding` test to assert `Unyielding`.
- Add a unit test: `spawn_in_unyielding_fn_is_valid` — confirms no error.

**interpreter.rs (unyielding path):**
- Remove the error arm for `Expression::Spawn` in `eval_expression_unyielding`.
- The unyielding path handles spawn by calling the target function's shim (which returns
  `Value::Future` for yielding callees) and returning the future. The caller does not await.

**proto/ish-tests/functions/analyzer.sh:**
- Replace "fn with spawn classified yielding" with "fn with spawn classified unyielding":
  ```bash
  output=$(run_ish 'async fn inner() { return 5 }
                    fn wrapper() { return spawn inner() }
                    println(is_yielding(wrapper))')
  assert_output "fn with spawn classified unyielding" "false" "$output"
  ```

**proto/ish-tests/concurrency/unyielding_context.sh:**
- Remove the `@[unyielding] fn bad() { spawn some_fn() }` test entirely.
  It tested invalid behavior; no replacement is needed.

---

## Feature 3: Fix Undefined Function Call Behavior

The analyzer at `proto/ish-vm/src/analyzer.rs:215-220` silently treats undefined function
calls as unyielding:

```rust
Err(_) => {
    // Undefined variable — treat as unyielding (conservative).
    // This handles forward references and variables not yet in scope.
    // Known limitation: forward references to yielding functions
    // may be misclassified as unyielding.
}
```

The accepted stubbed-analyzer proposal (Decision 10) specified that undefined function calls
should be an error. The implementation deviated. The deviation must be corrected.

### Issues to Watch Out For

- The unit test `call_to_undefined_fn_is_unyielding` (analyzer.rs:445) explicitly asserts
  the incorrect behavior. It must be replaced.
- The unit test `call_to_parameter_name_is_unyielding` (analyzer.rs:459) uses a child
  environment with parameter names pre-populated as `Value::Null`. This mechanism already
  correctly handles parameter calls without triggering the undefined error. No change needed.
- Forward references are explicitly not supported in the stub. Silently treating them as
  unyielding can cause misclassification; failing loudly is safer and was the intended design.
- Acceptance tests in `analyzer.sh` must include a test for this error case.

### Implementation

**analyzer.rs:**
- Replace the `Err(_)` arm in `expr_contains_yielding` (FunctionCall branch):
  ```rust
  Err(_) => {
      return Err(RuntimeError::system_error(
          ErrorCode::UndefinedVariable,
          &format!("undefined function '{}' — forward references are not supported", name),
      ));
  }
  ```
- Update unit test: `call_to_undefined_fn_is_unyielding` → `call_to_undefined_fn_is_error`,
  asserting `classify_function` returns `Err(...)`.

**proto/ish-tests/functions/analyzer.sh:**
- Add test:
  ```bash
  output=$(run_ish 'fn bad() { undefined_fn() }')
  assert_output_contains "undefined fn call errors at declaration" "E005" "$output"
  ```

---

## Feature 4: Fix Implied Await Scope

The yielding `FunctionCall` handler in `interpreter.rs` currently performs implied await
unconditionally when `call_function_inner` returns a `Value::Future`. The correct rule is:

> Implied await applies only when calling a **yielding** function. Calling an unyielding
> function that happens to return a `Value::Future` must not trigger implied await.

This matters for `apply`: `apply` is an unyielding compiled function. When called with a
yielding argument, its shim calls the yielding function's shim, which returns `Value::Future`.
`apply` returns that future as its own result. The current code then unconditionally awaits
it — wrong. The future should be returned as-is.

### Issues to Watch Out For

- The current cross-boundary yielding tests (`cross_boundary_yielding.sh`) assert the
  incorrect behavior (`type_of(apply(work, [10]))` → `"int"`). They must be corrected.
- After fixing implied await, getting the resolved value from `apply(async_fn, args)` requires
  the caller to explicitly await the returned future. Feature 5 (grammar expansion) is a
  prerequisite for clean test cases.
- The concurrency spec section on Implied Await requires updating.
- The distinction is between **calling a yielding function** (implied await applies) and
  **receiving a future from an unyielding function** (no implied await).

### Implementation

**interpreter.rs (yielding FunctionCall handler):**
- After calling `call_function_inner`, check the callee's `has_yielding_entry` before
  performing implied await:
  ```rust
  let result = call_function_inner(...)?;
  if func.has_yielding_entry == Some(true) {
      if let Value::Future(f) = result {
          return Ok(f.await?);
      }
  }
  Ok(result)
  ```

**docs/spec/concurrency.md:**
- Update the Implied Await section to clarify: implied await applies only when calling a
  yielding function. Unyielding functions returning futures do not trigger implied await.
- Add an example distinguishing the two cases.

**docs/architecture/vm.md:**
- Update the `apply` description and the FunctionCall yielding-path description.

---

## Feature 5: Expand Grammar to Allow `await expr`

Currently `await` and `spawn` require a function call as their operand:

```pest
await_op ~ call_expr
spawn_op ~ call_expr
```

This prevents patterns like:

```ish
let f = apply(async_fn, [args])
await f  // currently a parse error
```

The grammar is expanded so that `await` accepts any expression. At runtime, if the expression
does not evaluate to a `Value::Future`, ish throws E014 (`AwaitNonFuture`). The `Await` AST
node is restructured from `{callee, args}` to `{expr}`.

`spawn` is **not** expanded — spawning a variable-held function doesn't fit the semantics
cleanly (you'd need to call it anyway).

### Issues to Watch Out For

- The `Await` AST node currently has the shape `{ callee: Box<Expression>, args: Vec<Expression> }`
  (mirroring `FunctionCall`). The new shape `{ expr: Box<Expression> }` is a breaking AST
  change that touches the parser, all AST consumers (interpreter, analyzer, future tests),
  and the spec.
- `await func()` must remain valid. After the grammar change, `await func()` is parsed as
  `await` applied to a `FunctionCall` expression — no special handling needed.
- The E012 ("await applied to unyielding function") check currently happens before the call.
  With the new AST shape, E012 only applies when the `await`-ed expression is a `FunctionCall`
  to a known unyielding function. Awaiting a variable bypasses this check; E014 is the fallback.
- The analyzer must not treat the new `Await { expr }` form differently from the old form
  for yielding classification purposes — any `await` node still makes the enclosing function
  yielding.
- The spec currently says `await` and `spawn` "syntactically require a function call." That
  sentence must be updated for `await`.

### Implementation

**ish-parser:**
- Change `await_op` rule to `await_op ~ expr` (or an appropriate production that prevents
  left-recursive ambiguity).
- Update the `Await` AST node in `ish-ast`:
  ```rust
  Expression::Await { expr: Box<Expression> }
  ```
- Update `FunctionCall`-based E012 check: in the parser or interpreter, when the `await`-ed
  expression is a `FunctionCall`, check the callee's yielding classification.

**interpreter.rs (yielding path, `Expression::Await` handler):**
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
*(The `Value::Function` arm is an informative error for the `await fn_ref` case — not
passing arguments. The primary guard against `await unyielding_fn()` is the FunctionCall
check.)*

**docs/errors/INDEX.md and docs/spec/errors.md:**
- Add E014: `AwaitNonFuture` — `TypeError` — "`await` applied to a non-future value".
- Production site: `interpreter.rs` (Expression::Await, non-Future value).

**docs/spec/concurrency.md:**
- Update "Grammar Restrictions" section: `await` now accepts any expression. `spawn` still
  requires a function call.
- Add runtime behavior: if the awaited expression is not a `Value::Future`, E014 is thrown.

**proto/ish-tests/concurrency/spawn_await.sh:**
- Add tests covering simple identifiers, complex expressions, and error cases:
  ```bash
  # await a variable holding a future
  output=$(run_ish 'async fn work() { return 42 }
                    let f = spawn work()
                    println(await f)')
  assert_output "await variable resolves future" "42" "$output"

  # await a complex expression (e.g. list index, conditional, nested call)
  output=$(run_ish 'async fn work(x) { return x + 1 }
                    let fs = [spawn work(1), spawn work(2)]
                    println(await fs[0])')
  assert_output "await complex expression resolves future" "2" "$output"

  # await a non-future value → E014
  output=$(run_ish 'let x = 42
                    await x')
  assert_output_contains "await non-future throws E014" "E014" "$output"
  ```

---

## Feature 6: Fix Unyielding Context Acceptance Tests

The file `proto/ish-tests/concurrency/unyielding_context.sh` has two distinct problems:

1. All three tests call `some_fn()` which is not defined. The analyzer fails with E005
   (undefined variable) before it can detect the unyielding context violation.

2. The test "`@[unyielding]` fn body containing spawn → analyzer error at declaration" is
   semantically wrong. Unyielding functions are allowed to call `spawn`.

### Issues to Watch Out For

- Simply defining `some_fn` as an unyielding function reveals another dependency: `await
  some_fn()` requires `some_fn` to be yielding for the E012 check. The test scenario for
  "await in unyielding body" needs a **yielding** `some_fn`.
- The "yield in unyielding body" test (`@[unyielding] fn bad() { yield }`) needs no callee
  and is unaffected.

### Implementation

**proto/ish-tests/concurrency/unyielding_context.sh** — three scenarios:

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

---

## Feature 7: Fix Shell Integration Tests

The shell integration tests (`proto/ish-tests/concurrency/shell_integration.sh`) mark
functions using `$()` as `async`. This is redundant: command substitution (`$(...)`) is
classified as a yielding node by the analyzer, so any function containing `$()` is
automatically classified as yielding without requiring an explicit `async` keyword.

Leaving the `async` keyword in place gives a misleading impression that the keyword is
necessary. Removing it tests that the analyzer correctly classifies these functions.

### Implementation

**proto/ish-tests/concurrency/shell_integration.sh:**
- Remove `async` from `async fn work()` in the "shell command in awaited task" test.
- The function body uses `$(echo from_spawn)`, so the analyzer classifies it as yielding
  without `async`.

---

## Feature 8: Add Missing Analyzer Acceptance Tests

The analyzer classifies `Expression::CommandSubstitution` and `Statement::ShellCommand` as
yielding nodes (correctly, per the spec). Neither has an acceptance test.

### Implementation

**proto/ish-tests/functions/analyzer.sh:**
```bash
# function using command substitution is yielding
output=$(run_ish 'fn uses_subst() { let r = $(echo hello); return r }
                  println(is_yielding(uses_subst))')
assert_output "fn with command substitution is yielding" "true" "$output"

# function executing a shell command is yielding
output=$(run_ish 'fn runs_shell() { echo hello }
                  println(is_yielding(runs_shell))')
assert_output "fn with shell command is yielding" "true" "$output"
```

---

## Feature 9: Fix Cross-Boundary Yielding Tests

After Features 2, 4, and 5 are implemented, the cross-boundary yielding tests need a full
rewrite.

### Implementation

**proto/ish-tests/functions/cross_boundary_yielding.sh** (new content):

```bash
# apply(async_fn, args) returns a Future — no implied await (apply is unyielding)
output=$(run_ish 'async fn work(x) { return x + 1 }
                  println(type_of(apply(work, [10])))')
assert_output "apply async fn returns future" "future" "$output"

# The returned future can be awaited (requires Feature 5: await variable)
output=$(run_ish 'async fn work(x) { return x + 1 }
                  let f = apply(work, [10])
                  println(await f)')
assert_output "apply async fn future resolves correctly" "11" "$output"

# apply(unyielding_fn, args) returns the result directly
output=$(run_ish 'fn add(a, b) { return a + b }
                  println(apply(add, [3, 4]))')
assert_output "apply unyielding fn direct result" "7" "$output"

# apply is itself unyielding — is_yielding returns false
output=$(run_ish 'println(is_yielding(apply))')
assert_output "apply is unyielding" "false" "$output"
```

---

## Sequencing

Features have dependencies:

1. **Feature 1** (`is_yielding` builtin) — no dependencies; implement first.
2. **Feature 2** (spawn not yielding) — no dependencies; implement alongside Feature 1.
3. **Feature 3** (undefined fn errors) — no dependencies.
4. **Feature 5** (grammar expansion, `await expr`) — no dependencies on other features,
   but is a larger change.
5. **Feature 4** (implied await scope) — depends on Feature 5 for the test cases.
6. **Feature 6** (unyielding context tests) — depends on Feature 2.
7. **Feature 7** (shell integration tests) — no dependencies.
8. **Feature 8** (missing acceptance tests) — depends on Feature 1.
9. **Feature 9** (cross-boundary tests) — depends on Features 1, 2, 4, 5.

---

## Documentation Updates

| File | Changes |
|------|---------|
| [docs/spec/concurrency.md](../../spec/concurrency.md) | Update Implied Await (Feature 4). Update Grammar Restrictions: `await` accepts any expression (Feature 5). Document `await non-future` → E014 (Feature 5). |
| [docs/architecture/vm.md](../../architecture/vm.md) | Update `apply` description. Update FunctionCall yielding-path description (implied await scope). Clarify spawn in unyielding context. |
| [docs/errors/INDEX.md](../../errors/INDEX.md) | Add E014: `AwaitNonFuture` (Feature 5). |
| [docs/spec/errors.md](../../spec/errors.md) | Add E014 to the error code table (Feature 5). |
| [GLOSSARY.md](../../../GLOSSARY.md) | Update "implied await" definition to clarify it applies only to yielding function calls. |

---

## Referenced by

- [docs/project/rfp/analyzer-correctness.md](../rfp/analyzer-correctness.md)
- [docs/project/proposals/INDEX.md](INDEX.md)
