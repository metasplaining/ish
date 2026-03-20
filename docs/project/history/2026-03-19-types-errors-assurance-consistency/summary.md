---
title: "History: Types, Errors, and Assurance Ledger — Consistency Audit"
category: history
audience: [all]
status: draft
last-verified: 2026-03-19
depends-on: [docs/project/proposals/types-errors-assurance-consistency.md]
---

# Types, Errors, and Assurance Ledger — Consistency Audit

## Origin

This proposal began as a short prompt asking to audit types, errors, and the assurance ledger for consistency — determining which features were specified, which were implemented, which were tested, and which had gaps between them. The prompt recognized that these three feature areas are deeply intertwined and that many features depend on the assurance ledger, but not all were expected to be complete.

## v1 — Initial Audit (2026-03-19)

The first version performed a broad audit across three feature areas and uncovered significant inconsistencies:

The maturity matrix had at least four rows that were completely wrong — error handling, syntax/grammar, and the parser were all marked as not designed, not specified, not prototyped, and not tested, when in fact all three had accepted proposals, specifications, working prototypes, and partial test coverage. The roadmap listed error handling design as "in progress" when an accepted proposal already existed and the prototype had a working try/catch/finally/defer implementation with 16 test assertions.

The audit identified 8 features for analysis, ranging from quick documentation fixes (maturity matrix corrections, naming reconciliation) to substantial implementation work (assurance ledger runtime, type annotation enforcement, error code integration). Each feature was presented with alternatives, pros/cons analysis, and implementation details. Nine decision points were left open for the human reviewer.

## v2 — Decisions and Restructuring (2026-03-19)

The human reviewer made sweeping decisions that fundamentally reshaped the proposal's direction. The most significant insight was that the error handling system, originally specified before the assurance ledger existed, needed to be rebuilt on top of the ledger rather than alongside it.

**Error revolution.** The reviewer decided that errors should not be a separate concept with their own `RuntimeError` struct and `new_error()` function. Instead, errors are ordinary objects that happen to have an `Error` entry in the assurance ledger. The `Error` entry type requires a `message: String` property. A `SystemError` entry extends `Error` and adds a required error code. The language processor only throws SystemErrors. This means `throw { message: "bad" }` works because the ledger auto-adds the Error entry, while `throw { errno: 73 }` fails because the object can't satisfy the Error entry requirements — and the failure itself becomes a SystemError about the discrepancy. The `new_error()` builtin was marked for removal — it's unnecessary when errors are just objects with entries.

**Scope expansion.** The reviewer pulled five additional type features into scope: generic types, type narrowing, literal types, intersection types, and custom type guards. Rather than implementing these immediately, their specifications need to be refined first, with the open questions from types.md resolved as formal decision points. Crucially, the reviewer observed that many features originally specified as part of the type system (mutability enforcement, null safety, overflow configuration) are actually assurance ledger features that have nothing to do with typing — they belong in or are cross-referenced from the ledger spec.

**Type enforcement through the ledger.** The reviewer rejected the option to defer type annotation enforcement. Instead, live-audit type checking should be implemented as the `types(live)` feature in the assurance ledger. This means type checking, error validation, and other enforcement all flow through the same ledger audit pipeline.

**Naming conventions.** Rather than just picking snake_case or camelCase for one function name, the reviewer asked for a comprehensive naming convention for all ish code, taking a "no surprises" approach informed by popular language conventions (especially Rust and TypeScript). The v2 proposal researched eight languages and proposed snake_case for functions/variables, PascalCase for types, and SCREAMING_SNAKE_CASE for constants — matching Rust conventions and the existing prototype — but left the final decision to the reviewer.

**Phased implementation.** The reviewer directed that errors be pushed to later phases since they should make full use of the type and assurance ledger mechanisms once those are in place. The v2 proposal organized work into three phases: foundations (ledger runtime, naming, documentation fixes), types (spec refinement, live-audit implementation), and errors (entry-based error model, catalog expansion, spec consolidation).

**Process improvements.** The reviewer also identified that the maturity matrix was missing from the authority order — it was mentioned in CONTRIBUTING.md's change protocol but not in the numbered authority sequence used by the implement skill. The v2 proposal adds it at positions 3 and 12 (after roadmap updates, before specifications, and again after code/tests).

## v3 — Deep Type System Redesign (2026-03-19)

The third revision brought the most profound conceptual changes yet. Where v2 reshaped error handling around the ledger, v3 reshaped the entire type system.

**Literal types abolished in favor of value entries.** The reviewer's central insight was that ish doesn't need literal types at all. In TypeScript, `let x: 5 = 5` makes `5` the *type*. In ish, `let x = 5` means x has *type* `int` and a *value entry* of `5`. The reviewer identified three use cases for tracking specific values — reachability inference (can this branch execute?), never-type inference (are there no possible values?), and value constraint checking (is this value in the allowed set?). All three are handled by entries, not types. Research into TypeScript, Kotlin, Scala, and Haskell confirmed that all eleven major literal-type use cases from other languages (discriminated union narrowing, exhaustive matching, const assertions, function overload keying, etc.) fit the value-entry model without requiring literal-as-type semantics.

This led to a three-part value-entry taxonomy: *actual-value entries* (live audit — the concrete runtime value), *possible-values entries* (pre audit — all values a variable could hold across execution paths), and *allowed-values entries* (explicit constraints on permitted values). Type declarations serve double duty as structural type definitions *and* hooks for behavioral entries — a `type Direction = "north" | "south"` is both a structural union and an allowed-values constraint.

**Type declarations are indeterminate, not closed.** The reviewer corrected a fundamental assumption in the type spec: type declarations are neither open nor closed by default. They are *indeterminate* — a type declaration like `type Cat = Animal & { sound: String }` doesn't commit to whether extra properties are allowed. Only object *literals* are closed by default. The open/closed distinction becomes an entry in the ledger, resolved when values are assigned: `let x: Cat = { sound: "Meow" }` combines the indeterminate Cat declaration with the closed object literal, and the ledger determines the combined entry set.

**Variance is structurally determined.** The reviewer argued that variance annotations are meaningless in ish's structural type system. A `Cat` that structurally satisfies `Animal` is always assignable to `Animal` by structural subtyping — no covariance annotation needed. The only exception is the open/closed distinction: if `Animal` is closed, a `Cat` with extra properties is a discrepancy. The ledger handles this through entry checking, not variance annotations.

**Deferred type inference.** For unannotated bindings like `let x = 5`, the interpreter retains the string literal and defers parsing/conversion until the actual type is inferred from context. The compiler infers during pre-audit and widens if no inference is possible.

**Error hierarchy deepened.** Building on v2's Error-entry model, the reviewer added an intermediate `CodedError` layer. The hierarchy is now: Error (has `message: String`) → CodedError (has `code: String`) → SystemError (code is a well-known ish code) → domain subtypes (FileError → FileNotFoundError, TypeError, ArgumentError, etc.). Each leaf gets a specific error code. Ordinary user errors don't need codes (`throw { message: "bad" }` is just an Error), errors with user-defined codes are CodedErrors, and only the language processor creates SystemErrors.

**Type narrowing is ledger behavior.** The reviewer clarified that narrowing isn't a type-checker feature grafted onto an audit system — it's the natural consequence of the ledger maintaining entries. After every statement, the ledger creates a revised entry set for every value in scope. Type is just one entry among many. An `if is_type(x, "int")` narrows the type entry in that branch; when branches converge, entries are merged. This unified model means mutability tracking, null narrowing, value tracking, and type narrowing are all the same mechanism.

**Feature state naming challenged.** The reviewer found the `optional`/`live`/`pre` feature state names confusing because they conflate two independent dimensions: whether annotations are required and when checking happens. The v3 proposal analyzed three alternatives: keeping the current names with better documentation (A), splitting into orthogonal `type_annotations(optional|required)` and `type_audit(runtime|build)` dimensions (B), or collapsing to just `inferred`/`explicit` (C). The proposal recommends B but the decision remains open.

**Two decisions remain open.** The ledger datatype location (where should standards and entry types be defined — AST, VM, ish-on-VM, or hybrid?) and the feature state naming scheme are left for the reviewer. All other decisions from v1 and v2 are resolved, yielding 22 total decisions in the register.

## Acceptance (2026-03-19)

After three rounds of design, all 22 decisions were resolved and the proposal was accepted. The final state represents a comprehensive redesign of how types, errors, and the assurance ledger interact:

- The assurance ledger is the central enforcement mechanism for both types and errors. It maintains entry sets for every value after every statement — type checking, error validation, narrowing, and open/closed semantics all flow through the same ledger pipeline.
- Errors are ordinary objects with ledger entries, not special runtime constructs. The Error → CodedError → SystemError → domain-subtype hierarchy replaces the Rust `RuntimeError` struct and `new_error()` builtin.
- Literal types are replaced by value entries (actual/possible/allowed), which handle all the same use cases through the ledger rather than the type system.
- Type declarations are indeterminate (neither open nor closed), with the ledger resolving open/closed during assignment. Object literals are closed by default.
- Type feature states are split into two orthogonal dimensions: `type_annotations` (optional/required) and `type_audit` (runtime/build).
- Ledger datatypes use a hybrid architecture: AST for immutable declarations, a separate ledger module for audit logic, VM for wiring.
- Naming conventions formalized: `snake_case` for functions/variables, `PascalCase` for types, `SCREAMING_SNAKE_CASE` for constants.

Implementation proceeds in three phases: foundations (ledger runtime, documentation fixes, naming), types (spec refinement, live-audit, narrowing), and errors (entry-based model, catalog, spec consolidation).

---

## Referenced by

- [docs/project/history/INDEX.md](../INDEX.md)
