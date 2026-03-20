---
title: "Plan: Implementation Audit — Types, Errors, and Assurance Ledger"
category: plan
audience: [ai-dev]
status: completed
last-verified: 2026-03-20
depends-on:
  - docs/project/proposals/implementation-audit-types-errors-ledger.md
  - docs/spec/assurance-ledger.md
  - docs/spec/errors.md
  - docs/architecture/vm.md
---

# Plan: Implementation Audit — Types, Errors, and Assurance Ledger

*Derived from [implementation-audit-types-errors-ledger.md](../proposals/implementation-audit-types-errors-ledger.md) on 2026-03-20.*

## Overview

Fix bugs introduced during the types/errors/assurance-ledger consistency implementation. The core problem: the interpreter gates entry maintenance on the `types` feature, but entry maintenance is unconditional. Additionally: remove non-`Error` entry types from Rust (structural error hierarchy), fix defer scoping regression in errors.md, add ledger observability builtins, rewrite vacuous acceptance tests, and add spec clarity on maintenance-vs-auditing and assurance-level semantics.

## Requirements

Each requirement is a testable statement extracted from the accepted proposal.

- R1: Type narrowing (entry save/restore/merge/narrow) runs unconditionally in the `if` handler, regardless of active standard. (Decision 1)
- R2: Throw audit runs unconditionally in the `throw` handler. (Decision 2)
- R3: Throw audit auto-adds `@Error` entry to objects with `message: String`. Non-qualifying values are wrapped in a system error object. (Decision 3)
- R4: `audit_type_annotation` always checks present annotations for compatibility. Missing-annotation discrepancy only fires when `types` feature is `required`. (Decision 10)
- R5: `ledger_state(variable_name)` builtin returns a string representation of all entries on a variable. (Decision 5)
- R6: `has_entry(variable_name, entry_type)` builtin returns `true`/`false`. (Decision 5)
- R7: Type narrowing acceptance tests assert ledger state, not just printed constants. Tests work without any standard active. (Decision 6)
- R8: Only `@Error` remains as a predefined entry type. `CodedError`, `SystemError`, `TypeError`, `ArgumentError`, `FileError`, `FileNotFoundError`, `PermissionError` are removed from the Rust entry type registry. (Decisions 7, 8)
- R9: `docs/spec/errors.md` says `defer` is function-scoped, with cross-reference to defer-scoping proposal. (Decision 9)
- R10: `docs/spec/assurance-ledger.md` has an "Entry Maintenance vs. Auditing" subsection. (Decision 4)
- R11: `docs/spec/assurance-ledger.md` has an "Assurance Level Semantics" subsection explaining that low assurance defers to runtime, not disables. (Decision 11)
- R12: Built-in entry types table in `assurance-ledger.md` lists only `Error`, `Mutable`, `Type`, `Open`, `Closed`. (Decision 8)
- R13: Error hierarchy section in `errors.md` uses structural model with ish type definitions. (Decision 7)
- R14: `docs/errors/INDEX.md` hierarchy column updated — no `SystemError` parent chain, just `Error` or `Error (CodedError)`. (Decision 7)

## Authority Order

1. Specification docs
2. Architecture docs
3. Error catalog
4. Acceptance tests
5. Code (implementation)
6. Unit tests (cargo test)
7. Maturity matrix (if affected)
8. AGENTS.md (update test counts)
9. History
10. Index files

No glossary changes or roadmap entries needed — this is a bug-fix audit, not a new feature.

## TODO

### Checkpoint A: Specification Fixes

- [x] 1. **assurance-ledger.md — add "Entry Maintenance vs. Auditing"** — `docs/spec/assurance-ledger.md`. Add new subsection under Concepts (after "Discrepancies" ~L92, before "Entry Types" ~L98). Content per Issue 3: entry maintenance is always performed; auditing is governed by features.
- [x] 2. **assurance-ledger.md — add "Assurance Level Semantics"** — `docs/spec/assurance-ledger.md`. Add subsection (after "Entry Maintenance vs. Auditing"). Content per Issue 4: continuum from all-runtime to all-build-time; low assurance defers, not disables; streamlined has no features but still checks at runtime.
- [x] 3. **assurance-ledger.md — update built-in entry types table** — `docs/spec/assurance-ledger.md` ~L104-109. Remove `CodedError`, `SystemError` rows. Keep `Error` (requires `message: String`), `Mutable`, `Type`, `Open`, `Closed`.
- [x] 4. **errors.md — fix defer scoping** — `docs/spec/errors.md` ~L134. Change "scoped to the enclosing block, not the function" to "scoped to the enclosing function" with cross-reference to defer-scoping proposal. Per Issue 7.
- [x] 5. **errors.md — update error hierarchy to structural model** — `docs/spec/errors.md` ~L16-26. Replace inheritance table with structural definitions: `@Error` is the only entry type; `CodedError = Error & { code: String }`; leaf errors have specific code values; domain types are unions. Per Issue 6.
- [x] 6. **Verify spec internal consistency** — Read both updated spec files end-to-end. Ensure no remaining references to CodedError/SystemError as entry types. Ensure no references to "types feature required" for entry maintenance.

### Checkpoint B: Architecture and Error Catalog

- [x] 7. **vm.md — update for maintenance-vs-auditing principle** — `docs/architecture/vm.md`. Update "Assurance Ledger Runtime" section (~L155) to reflect: VM notifies ledger of events, ledger performs maintenance unconditionally and auditing per standard. Update "Throw and Try/Catch" section (~L90) to reflect structural error model (only `@Error` entry type, no catch-by-entry-type). Update "Error Handling" section (~L186) to note structural hierarchy.
- [x] 8. **errors/INDEX.md — update hierarchy column** — `docs/errors/INDEX.md`. Update "Hierarchy" column: all errors are just `Error` (with structural `code` property). Remove `→ CodedError → SystemError → ...` chains. Update "Domain Subtype" column: these are ish types, not entry types. Add note that domain classifications are structural (ish types, not entry types).

### Checkpoint C: Acceptance Tests

- [x] 9. **Rewrite type_narrowing.sh** — `proto/ish-tests/type_narrowing/type_narrowing.sh`. Remove `typed_std` standard from all tests. Use `ledger_state()` and `has_entry()` to assert narrowing entries (ExcludeNull, Type) inside branches. Keep behavioral assertions (print output) alongside ledger assertions. Per Issue 5.
- [x] 10. **Verify error_handling tests still make sense** — `proto/ish-tests/error_handling/throw_catch.sh`, `error_codes.sh`. Check for references to `SystemError`, `TypeError`, etc. as entry types. If catch clauses use type-based matching (they don't yet per the code), note what needs updating. No changes expected — catch is currently match-all.

### Checkpoint D: Code — Entry Type Registry

- [x] 11. **Remove non-Error entry types from registry** — `proto/ish-vm/src/ledger/entry_type.rs` ~L102-142. In `register_builtins`, remove registrations for: `CodedError`, `SystemError`, `TypeError`, `ArgumentError`, `FileError`, `FileNotFoundError`, `PermissionError`. Keep: `Error` (with `message: String` requirement), `Mutable`, `Type`, `Open`, `Closed`. Per Decision 8.
- [x] 12. **Check for is_subtype references to removed types** — Search codebase for `is_subtype("CodedError"`, `is_subtype("SystemError"`, `is_subtype("TypeError"` etc. Remove or replace with structural checks. Per the exploration, catch matching is currently all-match with no `is_subtype` calls, so this should be a no-op verification.

### Checkpoint E: Code — Interpreter Fixes

- [x] 13. **Remove types feature gate from if-handler** — `proto/ish-vm/src/interpreter.rs` ~L185-252. Remove the `types_active` check and the else branch (simple if/else without narrowing). Make narrowing unconditional: always analyze condition, save entries, narrow, execute, restore, merge. Per Decision 1.
- [x] 14. **Remove types feature gate from throw handler** — `proto/ish-vm/src/interpreter.rs` ~L329-345. Remove the `active_features().contains_key("types")` check. Make throw audit unconditional. Per Decision 2.
- [x] 15. **Implement full throw audit** — `proto/ish-vm/src/interpreter.rs` ~L333-345. Replace simple `has_message` check with full throw audit: (a) Object with `message: String` → auto-add `@Error` entry if not present. (b) Object without `message: String` → wrap in system error object `{ message: "throw audit: ...", code: "E001", original: <value> }` with `@Error` entry. (c) Non-object → wrap as (b). Per Decision 3.
- [x] 16. **Refactor audit_type_annotation** — `proto/ish-vm/src/interpreter.rs` ~L996. Remove the early return when `types` feature is absent. If annotation is present: always check compatibility. If annotation is absent: only report discrepancy when `types` feature is `required`. Per Decision 10.

### Checkpoint F: Code — Ledger Observability Builtins

- [x] 17. **Add ledger_state and has_entry builtin stubs** — `proto/ish-vm/src/builtins.rs` ~L28-46. Add `ledger_state` and `has_entry` to the ledger query stubs section, following the same pattern as existing stubs (return error saying must be intercepted by interpreter). Per Decision 5.
- [x] 18. **Add VM interception for ledger_state** — `proto/ish-vm/src/interpreter.rs`. In the function call handler where other ledger stubs are intercepted, add interception for `ledger_state(variable_name)`: look up the variable in the current environment, get its entries from the ledger, format as string (e.g., `"Type(i32), ExcludeNull"`), return as `Value::String`.
- [x] 19. **Add VM interception for has_entry** — `proto/ish-vm/src/interpreter.rs`. Add interception for `has_entry(variable_name, entry_type)`: look up the variable, check if it has an entry of the given type name, return `Value::Boolean`.

### Checkpoint G: Build and Test

- [x] 20. **cargo build --workspace** — `proto/`. Must compile cleanly.
- [x] 21. **cargo test --workspace** — `proto/`. All unit tests must pass.
- [x] 22. **bash ish-tests/run_all.sh** — `proto/`. All acceptance tests must pass, including rewritten narrowing tests.
- [x] 23. **Manual verification** — Run a quick inline test to verify narrowing works without a standard: `ish-shell -c 'let x: i32 | null = 42\nif x != null { println(ledger_state("x")) }'` should show narrowing entries.

### Checkpoint H: Documentation Cleanup

- [x] 24. **AGENTS.md — update test counts** — `AGENTS.md`. Update the test count numbers after any test changes.
- [x] 25. **Update Referenced by sections** — All modified files. Add cross-references where missing.
- [x] 26. **Update history** — `docs/project/history/2026-03-20-implementation-audit-types-errors-ledger/summary.md`. Add implementation narrative.
- [x] 27. **Update plans INDEX.md** — `docs/project/plans/INDEX.md`. Add this plan, mark as completed.

## Reference

### Key Source Locations

| File | Line(s) | What |
|------|---------|------|
| `proto/ish-vm/src/interpreter.rs` | 185-252 | If-handler with `types_active` gate |
| `proto/ish-vm/src/interpreter.rs` | 329-345 | Throw handler with `types` feature gate |
| `proto/ish-vm/src/interpreter.rs` | 366-378 | Catch clause matching (currently all-match) |
| `proto/ish-vm/src/interpreter.rs` | 996+ | `audit_type_annotation` with early return |
| `proto/ish-vm/src/ledger/entry_type.rs` | 102-142 | `register_builtins` — entry types to remove |
| `proto/ish-vm/src/ledger/entry_type.rs` | 91-99 | `is_subtype` method |
| `proto/ish-vm/src/builtins.rs` | 28-46 | Ledger query stubs |
| `docs/spec/assurance-ledger.md` | 92 | "Discrepancies" heading (insert after) |
| `docs/spec/assurance-ledger.md` | 98 | "Entry Types" heading (insert before) |
| `docs/spec/assurance-ledger.md` | 104-109 | Built-in entry types table |
| `docs/spec/errors.md` | 16-26 | Error hierarchy section |
| `docs/spec/errors.md` | 134 | Defer scoping (wrong) |
| `docs/architecture/vm.md` | 90, 155, 186 | Throw/catch, ledger runtime, error handling sections |
| `docs/errors/INDEX.md` | 28-37 | Error codes table with hierarchy column |

### Existing Test Helpers

`proto/ish-tests/lib/test_lib.sh` already has `assert_output_contains` — no need to add `assert_contains`.

### Throw Audit Rules (from accepted proposal)

1. Object with `message: String` → auto-add `@Error` entry if not present.
2. Object without `message: String` → wrap in `{ message: "throw audit: thrown value does not qualify as an error", code: "E001", original: <value> }` with `@Error` entry.
3. Non-object value → wrap as rule 2.

### Structural Error Hierarchy (for spec, not Rust)

```ish
type CodedError = Error & { code: String }
type FileNotFoundError = CodedError & { code: "E008" }
type PermissionError = CodedError & { code: "EXXX" }
type TypeError = CodedError & { code: "E004" }
type ArgumentError = CodedError & { code: "E003" }
type FileError = FileNotFoundError | PermissionError
type SystemError = TypeError | ArgumentError | FileError | ...
```

### Entry Types Remaining in Rust

After cleanup, `register_builtins` should register only:
- `Error` (requires `message: String`)
- `Mutable`
- `Type`
- `Open`
- `Closed`

---

## Referenced by

- [docs/project/plans/INDEX.md](INDEX.md)
