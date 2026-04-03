---
title: "RFP: Stubbed Analyzer and Yielding/Unyielding Function Refactoring"
category: rfp
audience: [ai-dev]
status: draft
last-verified: 2026-04-03
depends-on:
  - docs/spec/concurrency.md
  - docs/architecture/vm.md
  - docs/project/proposals/concurrency-correctness.md
  - docs/project/proposals/runtime-extraction.md
---

# RFP: Stubbed Analyzer and Yielding/Unyielding Function Refactoring

## Problem Statement

The `PENDING_INTERP_CALL` mechanism is a defect. The code expects that the only place shims are ever called is from `call_function_inner`. This is not correct, however, because shims may also be called from compiled functions. The `apply` function exists for the sole purpose of testing this. But note how `apply` has been implemented as special-case logic in `call_function_inner` — the agent implementing the plan did this because it couldn't figure out how to make the system work if `apply` was a normal compiled function.

## Requested Changes

### 1. Remove the PENDING_INTERP_CALL Mechanism

Remove the `PENDING_INTERP_CALL` thread-local and the `InterpCall` struct. Fix `apply` so that it is a normal function that calls whatever function is passed into it. Expand the test set to include passing a yielding function to `apply` and verifying that it returns a `Value::Future`.

### 2. Create a Stubbed Code Analyzer

Before the function declaration mechanism can be fixed, we need a code analyzer. Create it in a new `analyzer.rs` file. For now, the analyzer has one job: at the time that a function is declared, determine whether the function is yielding or unyielding. It does this by walking the AST until it finds a statement that forces the function to be either yielding or unyielding. If it walks the whole AST and doesn't find either, then the function is implicitly unyielding.

This is just a stub implementation — it doesn't properly allow for automatic yielding. That will be addressed later.

### 3. Remove the Ambiguous Case

The code analyzer allows us to remove the ambiguous case. Every function will now be known to be yielding or unyielding. Remove the code that supports the ambiguous case (`has_yielding_entry: None`), and the documentation that describes it.

### 4. Yielding/Unyielding Execution Context

Modify the interpreter so that it stores state about whether the currently executing code is in a yielding context or an unyielding context. Propose alternative mechanisms for doing this.

### 5. Split eval_expression and exec_statement

Replace the existing `eval_expression` and `exec_statement` functions with new functions:

- `eval_expression_yielding`
- `eval_expression_unyielding`
- `exec_statement_yielding`
- `exec_statement_unyielding`

The yielding versions will basically be what we have now. The unyielding versions will return a `Value` or `Result` rather than a future, and will not include the outer task wrapping the whole function body.

To avoid repeating ourselves, extract the implementation block for each case of the match into a reusable function. The reusable functions will be used in both the yielding and unyielding versions.

### 6. Fix Function Declaration

When all that is done, properly implement function declaration so that it either creates a shim that calls `exec_statement_yielding` or one that calls `exec_statement_unyielding` depending on whether the function is yielding or unyielding. After all that, do we also need to have yielding and unyielding shim variants?

## Referenced by

- [docs/project/proposals/stubbed-analyzer.md](../proposals/stubbed-analyzer.md)
