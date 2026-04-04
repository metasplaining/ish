---
title: "History: Stubbed Analyzer and Yielding/Unyielding Function Refactoring"
category: history
audience: [all]
status: draft
last-verified: 2026-04-03
depends-on:
  - docs/project/proposals/stubbed-analyzer.md
  - docs/project/rfp/stubbed-analyzer.md
---

# Stubbed Analyzer and Yielding/Unyielding Function Refactoring

## Summary

This proposal addresses a fundamental architectural defect in the ish prototype: the `PENDING_INTERP_CALL` thread-local mechanism that bridges interpreted and compiled function execution. The mechanism was introduced during the runtime extraction work as a way to make interpreted function shims work within the synchronous `Shim` type signature, but it only functions correctly when shims are called from `call_function_inner`. The `apply` builtin — intended to test cross-boundary function calls — had to be implemented as a special-case intercept rather than as a normal compiled function, proving the defect.

The initial proposal (v1) analyzed six interconnected features needed to fix the architecture: a stubbed code analyzer for yielding classification, removal of the ambiguous `has_yielding_entry: None` case, replacement of `PENDING_INTERP_CALL` with self-contained shims, splitting the interpreter into yielding/unyielding execution paths, and fixing function declaration and `apply`. Nine decisions were presented with recommendations.

The reviewer accepted most recommendations but made two significant departures. First, the reviewer identified a missing analysis case: function calls without explicit `await` or `spawn` still cause yielding through the implied-await mechanism. This means the analyzer must check function calls against already-declared functions and mark the caller as yielding if the callee is yielding. This has deep implications: the analyzer needs VM environment access (reinforcing the `ish-vm` placement), functions must be defined in order (no forward references), and call cycles are not supported. The reviewer accepted this as a known limitation of the stub, to be fixed later.

Second, the reviewer rejected the recommendation to introduce a `YieldingClassification` enum to replace `Option<bool>` on `IshFunction.has_yielding_entry`. The reasoning was that the data structure needs a more significant refactoring later, and there is no point doing half the job now. Instead, `Option<bool>` is kept with `None` treated as unyielding rather than ambiguous.

The v2 revision incorporated all decisions: a new Decision 10 was added for the function-call analysis requirement, the analyzer's proposed implementation was updated to take an `Environment` parameter and return `Result` (to report undefined function errors), Feature 2 was simplified to retain `Option<bool>`, and all inline decision annotations were removed.

The proposal was accepted as v3 on 2026-04-03. The accepted version removed alternatives analysis, presenting only the chosen designs as settled fact. All 10 decisions were finalized. An implementation plan was generated to execute the six features in authority order.

## Implementation

The implementation plan organized 28 TODOs across 8 phases, following the authority order: documentation first, then acceptance tests, then code (analyzer, ambiguous case removal, execution variants, self-contained shims), then unit tests, and finally cleanup.

The documentation phase established the two-state yielding model in the concurrency spec and architecture docs, removing all references to the ambiguous `None` case and `PENDING_INTERP_CALL`.

The acceptance tests were written before the code changes, covering analyzer classification (6 tests), cross-boundary yielding through `apply` (3 tests), and unyielding context errors (3 tests). Several of these tests needed revision during implementation as the interaction between implied await and the new architecture became clearer.

The analyzer (`analyzer.rs`) was implemented as a straightforward AST walker. One deviation from the plan emerged immediately: the plan specified that undefined function calls should error, but this was relaxed to treat them as unyielding (conservative) to handle forward references gracefully. The analyzer classifies by checking for yielding nodes — `await`, `spawn`, `yield`, shell commands, command substitutions — and by looking up callees in the environment to detect yielding propagation through implied await.

The split into yielding and unyielding execution paths added roughly 400 lines to the interpreter. The unyielding variants (`exec_statement_unyielding`, `eval_expression_unyielding`) are synchronous — no `async`, no `Pin<Box<dyn Future>>`, no `.await`. They error on async operations (await, spawn, yield, shell commands, command substitution) with descriptive error codes.

The most architecturally significant change was making function shims self-contained. In the old architecture, shims stored a `PENDING_INTERP_CALL` describing what to execute, and `call_function_inner` retrieved and executed it asynchronously. In the new architecture, yielding shims capture the VM, body, environment, and parameters, then call `tokio::task::spawn_local` to execute the body asynchronously, returning a `Value::Future` wrapping the `JoinHandle`. Unyielding shims capture the same data and execute the body synchronously, returning the result directly. This made `call_function_inner` synchronous — it simply does arity checking, parameter auditing, ledger intercepts, and calls the shim. The `InterpCall` struct, `PENDING_INTERP_CALL` thread-local, and `builtin_apply` special-case intercept were all deleted.

The `apply` builtin was rewritten as a normal compiled function: it extracts the function value and argument list, calls `(f.shim)(&arg_list)`, and returns whatever the shim returns. No special handling required — proving the architecture works.

A subtle interaction emerged between `apply` and implied await. In the yielding `FunctionCall` handler, if `call_function_inner` returns a `Value::Future`, the interpreter automatically awaits it. This means `apply(async_fn, [args])` transparently resolves through implied await — the user gets the value directly, not a Future. The acceptance tests were updated to reflect this: `type_of(apply(async_fn, [10]))` returns `"int"`, not `"future"`. Similarly, `await apply(...)` cannot work because `apply` is unyielding (E012).

An analyzer gap was discovered during acceptance testing: `Statement::ShellCommand` was not classified as a yielding node, causing functions with shell commands to be misclassified as unyielding. The fix was straightforward — one line in the analyzer's `contains_yielding_node` match.

Another test issue revealed a grammar restriction: `await` requires a function call (`await func()`), not an arbitrary expression (`await variable`). A test that stored a spawned future in a variable and attempted to await the variable had to be restructured to verify classification through `type_of` instead.

## Referenced by

- [docs/project/proposals/stubbed-analyzer.md](../../proposals/stubbed-analyzer.md)
- [docs/project/history/INDEX.md](../INDEX.md)
