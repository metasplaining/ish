---
title: "Plan: Prototype Code Quality Refactoring"
category: plan
audience: [ai-dev]
status: completed
last-verified: 2026-04-06
depends-on:
  - docs/project/proposals/refactoring-prototype.md
  - proto/ish-vm/src/interpreter.rs
  - proto/ish-vm/src/reflection.rs
  - proto/ish-vm/src/builtins.rs
  - proto/ish-parser/src/ast_builder.rs
---

# Plan: Prototype Code Quality Refactoring

*Derived from [refactoring-prototype.md](../../proposals/refactoring-prototype.md) on 2026-04-06.*

## Overview

A code quality refactoring of the `proto/` codebase targeting six findings from the
`/propose-refactoring` scan: one exit-code constant, one brace-scanning helper extraction,
two builtin-registration helper additions, four non-structural panic fixes in the parser,
and six shared helper extractions from the yielding/unyielding interpreter pair. No
language-visible behaviour changes. No new public API.

## Requirements

- `MISSING_EXIT_CODE` constant replaces bare `-1` in both exit-code unwrap sites.
- `scan_to_close_brace` replaces duplicate brace-scanning loops in `interpolate_shell_quoted`; the inline `.unwrap()` on `name.chars().next()` is removed.
- `arity(name, args, n)` and `new_builtin(name, f)` helpers are available in `builtins.rs`; all `register_*` functions use them.
- `simple_ast_builtin(name, arity, fields)` is added to `reflection.rs`; all structurally simple builtins in `register_ast_builtins` use it (exceptions: `ast_literal`, `ast_param`, `ast_assign_target_var`).
- Integer overflow in `build_match_pattern` returns a `ParseError` rather than panicking.
- Float overflow in `build_match_pattern` returns a `ParseError` rather than panicking.
- `lines.last()` in `strip_triple_quote_literal` uses `.unwrap_or` instead of `.unwrap()`.
- `value.unwrap()` in `build_var_decl` is replaced with `?` propagation.
- A new acceptance test confirms that an integer literal too large for i64 produces an error message containing "overflows".
- Six shared helpers (`eval_literal`, `eval_unary_op`, `apply_property_read`, `apply_index_read`, `apply_property_write`, `apply_index_write`) exist in `interpreter.rs` and are called from both yielding and unyielding execution paths.
- `cargo test --workspace` passes after every phase.

## Phase Dependency Graph

```
Phase 1 (L1)  ──┐
                 ├──► Phase 6 (H1)  [all touch interpreter.rs]
Phase 2 (M3)  ──┘

Phase 3 (M2)  [builtins.rs — independent]

Phase 4 (M1)  [reflection.rs — independent]

Phase 5 (H2)  [ast_builder.rs — independent]

Phase 6 (H1)  [interpreter.rs — do last; largest change]
```

Phases 1–5 are independent of each other. Phase 6 must follow Phase 1 and Phase 2 (same
file; apply in sequence to avoid conflicts).

## Phases

| Phase | Feature | File(s) | Risk |
|-------|---------|---------|------|
| [1](phase-1.md) | L1 — exit-code constant | `interpreter.rs` | None |
| [2](phase-2.md) | M3 — `scan_to_close_brace` | `interpreter.rs` | Low |
| [3](phase-3.md) | M2 — `arity` + `new_builtin` | `builtins.rs` | Low |
| [4](phase-4.md) | M1 — `simple_ast_builtin` | `reflection.rs` | Low |
| [5](phase-5.md) | H2 — non-structural unwrap fixes | `ast_builder.rs` | Low |
| [6](phase-6.md) | H1 — six interpreter helpers | `interpreter.rs` | Medium |

## Context Files

- [context/helper-signatures.md](context/helper-signatures.md) — exact function signatures
  and bodies for all new helpers (verbatim from proposal; verified against source)
- [context/reflection-exceptions.md](context/reflection-exceptions.md) — the three
  `ast_*` builtins that must NOT be converted to `simple_ast_builtin`

## Authority Order for This Refactoring

This is a pure code refactoring with no spec, docs, or user-visible changes. The relevant
authority order is:

1. Code (implementation) — all six phases
2. Unit tests — phase 5 only (new acceptance test)
3. Roadmap / history — after all phases complete

## Referenced by

- [docs/project/proposals/refactoring-prototype.md](../../proposals/refactoring-prototype.md)
