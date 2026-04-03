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

## Referenced by

- [docs/project/proposals/stubbed-analyzer.md](../../proposals/stubbed-analyzer.md)
- [docs/project/history/INDEX.md](../INDEX.md)
