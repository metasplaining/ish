---
title: Analyzer Correctness Fixes
category: rfp
audience: [ai-dev]
status: active
date: 2026-04-04
depends-on:
  - docs/spec/concurrency.md
  - docs/architecture/vm.md
  - docs/project/proposals/stubbed-analyzer.md
  - proto/ish-vm/src/analyzer.rs
  - proto/ish-tests/functions/analyzer.sh
  - proto/ish-tests/concurrency/unyielding_context.sh
  - proto/ish-tests/concurrency/shell_integration.sh
  - proto/ish-tests/functions/cross_boundary_yielding.sh
---

# RFP: Analyzer Correctness Fixes

*Cleaned-up version of `analyzer_correctness` prompt file.*

---

## Background

The stubbed analyzer was implemented as part of the yielding/unyielding function refactoring
(see [stubbed-analyzer proposal](../proposals/stubbed-analyzer.md)). The implementation
introduced several correctness errors in both the analyzer itself and the acceptance tests.
The implementation summary (in [history](../history/2026-04-03-stubbed-analyzer/summary.md))
documents several deviations and discoveries that now need to be addressed.

---

## Issues

### 1. Add an `is_yielding` builtin

The acceptance tests for the analyzer are difficult to understand. We should have an
`is_yielding` builtin to directly determine whether a function is yielding, and use it in the
analyzer acceptance tests, rather than trying to infer whether a function is yielding from
how it behaves.

### 2. Broken test: "function containing spawn is yielding"

The analyzer acceptance test "function containing spawn is yielding" is broken. Spawn calls
explicitly do **not** make a function yielding. That is what the test is supposed to be
testing. Instead, it tests whether `await` can be applied to an unyielding function that
returns a future.

### 3. Command substitution should make a function yielding

Using command substitution should make a function yielding, and the analyzer acceptance tests
should have a test for it.

### 4. Remove unnecessary `async` from shell integration tests

The shell integration acceptance tests were changed to add an explicit `async` keyword to
functions using command substitution. That should be unnecessary and should be removed.

### 5. Unyielding context tests reference undefined `some_fn`

The unyielding context acceptance tests reference an undefined function `some_fn`. This would
cause the analyzer to fail regardless of the scenario being tested. The tests should be fixed
so that the function is defined.

### 6. Invalid test: `@[unyielding]` function containing spawn

The unyielding context acceptance test "`@[unyielding]` function body containing spawn →
analyzer error at declaration" is invalid. Unyielding functions should be able to call
`spawn`.

### 7. Undefined function calls should be an error

The implementation summary says:

> "the plan specified that undefined function calls should error, but this was relaxed to
> treat them as unyielding (conservative) to handle forward references gracefully."

The plan was correct. The code needs to be changed so that the analyzer treats undefined
function calls as an error. An acceptance test for this is needed.

### 8. Implied await must not apply to unyielding functions returning a future

The implementation summary says:

> "A subtle interaction emerged between `apply` and implied await. In the yielding
> `FunctionCall` handler, if `call_function_inner` returns a `Value::Future`, the interpreter
> automatically awaits it. This means `apply(async_fn, [args])` transparently resolves
> through implied await — the user gets the value directly, not a Future. The acceptance
> tests were updated to reflect this: `type_of(apply(async_fn, [10]))` returns `"int"`, not
> `"future"`. Similarly, `await apply(...)` cannot work because `apply` is unyielding
> (E012)."

That is incorrect and all needs to be fixed. Implied await should apply to yielding functions
only. It should not apply to unyielding functions that return a future. This behavior is
subtle, and we should make sure to document and test it thoroughly.

### 9. Shell commands as yielding nodes: document correctly

The implementation summary says:

> "An analyzer gap was discovered during acceptance testing: `Statement::ShellCommand` was
> not classified as a yielding node, causing functions with shell commands to be
> misclassified as unyielding. The fix was straightforward — one line in the analyzer's
> `contains_yielding_node` match."

This is the correct behavior. Shell commands should be treated as yielding. Make sure it is
documented correctly.

### 10. Expand grammar to allow `await variable`

The implementation summary says:

> "Another test issue revealed a grammar restriction: `await` requires a function call
> (`await func()`), not an arbitrary expression (`await variable`). A test that stored a
> spawned future in a variable and attempted to await the variable had to be restructured to
> verify classification through `type_of` instead."

We should expand the grammar to allow awaiting a variable. ish should throw an error when
attempting to await a variable that is not of type `future`.

### 11. Cross-boundary yielding tests are broken

The cross-boundary yielding tests are broken. The issues are all ones previously mentioned.
The tests should be carefully analyzed and fixed.

### 12. Spawn expressions incorrectly classified as yielding

The analyzer classifies spawn expressions as yielding. This is incorrect.
