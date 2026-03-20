---
title: "Plan: Types, Errors, and Assurance Ledger — Consistency"
category: plan
audience: [ai-dev]
status: completed
last-verified: 2026-03-19
depends-on:
  - docs/project/proposals/types-errors-assurance-consistency.md
  - docs/spec/types.md
  - docs/spec/assurance-ledger.md
  - docs/spec/syntax.md
  - docs/errors/INDEX.md
  - docs/architecture/vm.md
---

# Plan: Types, Errors, and Assurance Ledger — Consistency

*Derived from [types-errors-assurance-consistency.md](../proposals/types-errors-assurance-consistency.md) on 2026-03-19.*

## Overview

Resolve all inconsistencies between the type system, error handling, and assurance ledger specifications, documentation, and implementation. This includes fixing stale documentation (maturity matrix, roadmap), establishing naming conventions, building the assurance ledger runtime, implementing live-audit type checking and narrowing, overhauling error representation to use ledger entries, and comprehensively rewriting the type and ledger specs to reflect the new conceptual model. Work proceeds in three phases: foundations, types, errors.

## Requirements

Extracted from the accepted proposal. Each requirement is a testable statement.

### Foundations (Phase 1)
- R1.1: Maturity matrix rows for error handling, syntax/grammar, parser, and assurance ledger reflect actual state.
- R1.2: Roadmap lists "Error handling design" as Completed and "Types, errors, and assurance ledger consistency" as In Progress.
- R1.3: Authority order in CONTRIBUTING.md and copilot-instructions.md includes maturity matrix at positions 3 and 12.
- R1.4: Implement skill includes maturity matrix update step.
- R1.5: `docs/spec/syntax.md` has a naming conventions section specifying `snake_case`, `PascalCase`, `SCREAMING_SNAKE_CASE` rules.
- R1.6: `docs/spec/types.md` uses `is_type(value, type)` and `validate(type, value)` (not `isType`/`validate(t,i)`).
- R1.7: User guide and AI guide document naming conventions.
- R1.8: Ledger engine exists with `StandardRegistry`, `EntryTypeRegistry`, stateless `audit_statement()`, and `AuditResult` types.
- R1.9: VM integration provides standard scope stack, entry tracking on values, and audit bridge.
- R1.10: Built-in standards (`streamlined`, `cautious`, `rigorous`) and built-in entry types (`Error`, `CodedError`, `SystemError`, `Mutable`, `Type`) are pre-registered at startup.
- R1.11: Acceptance tests exist for: custom standard definition/application, feature state queries, entry type validation, built-in standards, standard inheritance, standard scope push/pop.
- R1.12: Acceptance tests exist for `with` blocks: basic usage, error during with, nested, combined with defer.

### Types (Phase 2)
- R2.1: `docs/spec/types.md` comprehensively rewritten: open/closed defaults corrected (type declarations indeterminate, object literals closed), value-entry model replaces literal types, generic types refined (variance structurally determined), intersection types specified, type narrowing cross-referenced to ledger, terminology clarified per Decision 22.
- R2.2: `docs/spec/assurance-ledger.md` comprehensively rewritten: feature state naming uses separate dimensions (`type_annotations`/`type_audit`), `Open`/`Closed` entry types added, type checking documented as ledger feature, type narrowing documented as entry maintenance behavior.
- R2.3: Type compatibility checking implemented in the ledger engine (simple, union, optional, object structural, function, list, tuple, intersection types).
- R2.4: Type audit wired into the ledger for assignment, function call, and return statements.
- R2.5: Type narrowing implemented as entry set maintenance: `is_type()` narrows in branch, null comparisons narrow, entries restored/merged after branches.
- R2.6: Acceptance tests for type checking: correct/incorrect annotations, function param/return types, union/optional/object matching, intersection types.
- R2.7: Acceptance tests for narrowing: `is_type` narrowing, null exclusion, entry restoration, nested narrowing.

### Errors (Phase 3)
- R3.1: `docs/spec/errors.md` exists with Error/CodedError/SystemError hierarchy, throw/catch/finally/defer semantics, ledger integration, error union types, `undeclared_errors` feature states, link to catalog.
- R3.2: Error content moved from `types.md` to `errors.md`.
- R3.3: `Error`, `CodedError`, `SystemError`, and domain subtypes (`FileError`, `TypeError`, `ArgumentError`, etc.) registered as built-in entry types.
- R3.4: Rust `RuntimeError` replaced — interpreter uses ish objects with `SystemError` entries for control flow.
- R3.5: `new_error()` removed. `is_error()` checks for `Error` entry. `error_message()` reads `message` property. `error_code()` added (reads `code` property).
- R3.6: Throw audits argument via ledger: auto-add `Error`/`CodedError` entry if qualifies, raise discrepancy if not.
- R3.7: All interpreter error creation sites produce `SystemError` objects with well-known codes.
- R3.8: Error catalog expanded with all error codes, domain subtypes, and hierarchy.
- R3.9: Acceptance tests for each error code, throw audit behavior, entry-based error identity.

## Authority Order

This plan uses the 14-step authority order established by this proposal (with maturity matrix added at positions 3 and 12).

1. GLOSSARY.md (if new terms)
2. Roadmap (set to "in progress")
3. Maturity matrix (update affected rows)
4. Specification docs
5. Architecture docs
6. User guide / AI guide
7. Agent documentation (AGENTS.md, skills, CONTRIBUTING.md, copilot-instructions.md)
8. Acceptance tests
9. Code (implementation)
10. Unit tests
11. Roadmap (set to "completed")
12. Maturity matrix (update affected rows)
13. History
14. Index files

---

## TODO

### Phase 1 — Foundations

- [x] 1. **GLOSSARY: Add new terms** — `GLOSSARY.md`
  - Add: `value entry`, `actual-value entry`, `possible-values entry`, `allowed-values entry`, `CodedError`, `SystemError`, `domain error subtype`, `intersection type`, `type narrowing`, `indeterminate` (for type openness context)
  - Update existing `entry type` definition to mention Error/CodedError/SystemError hierarchy
  - Update existing `feature state` definition to mention separate dimensions (`type_annotations`/`type_audit`)

- [x] 2. **ROADMAP: Set consistency work to "In Progress"** — `docs/project/roadmap.md`
  - Move "Error handling design" from In Progress to Completed (note: "design complete; implementation pending entry-based error model")
  - Add "Types, errors, and assurance ledger consistency" to In Progress

- [x] 3. **MATURITY: Fix inaccurate rows** — `docs/project/maturity.md`
  - Error handling: Designed ✅, Spec partial, Proto ✅, Tests partial, Stable ❌
  - Syntax / grammar: Designed ✅, Spec ✅, Proto ✅, Tests partial, Stable ❌
  - Parser: Designed ✅, Spec partial, Proto ✅, Tests partial, Stable ❌
  - Assurance Ledger: Designed ✅, Spec ✅, Proto partial, Tests ❌, Stable ❌

- [x] 4. **SPEC: Add naming conventions to syntax.md** — `docs/spec/syntax.md`
  - Add "Naming Conventions" section with table: variables (`snake_case`), functions (`snake_case`), types (`PascalCase`), constants (`SCREAMING_SNAKE_CASE`), modules (`snake_case`), entry types (`PascalCase`), standards (`snake_case`), keywords (`lowercase`)
  - Include rationale (matches existing prototype, Rust conventions, no rename needed)

- [x] 5. **SPEC: Fix function names in types.md** — `docs/spec/types.md`
  - Change `isType(t, i)` → `is_type(value, type)`
  - Change `validate(t, i)` → `validate(type, value)`
  - Change any other camelCase function references to `snake_case`

- [x] 6. **SPEC: Update assurance-ledger.md feature state naming** — `docs/spec/assurance-ledger.md`
  - Replace `optional`/`live`/`pre` single-dimension states with two-dimension model: `type_annotations` (`optional` | `required`) and `type_audit` (`runtime` | `build`)
  - Update all examples that reference old naming

- [x] 7. **USER GUIDE: Add naming conventions** — `docs/user-guide/language-basics.md`
  - Add naming conventions section with the convention table and examples

- [x] 8. **AI GUIDE: Add naming conventions** — `docs/ai-guide/orientation.md`
  - Add naming conventions section with the convention table

- [x] 9. **AGENT: Add maturity matrix to authority order** — `CONTRIBUTING.md`, `.github/copilot-instructions.md`
  - In both files, update the 12-step authority order to 14 steps:
    - Insert "Maturity matrix (update affected rows)" at position 3 (after Roadmap)
    - Insert "Maturity matrix (update affected rows)" at position 12 (after Unit tests, before History)

- [x] 10. **AGENT: Update implement skill** — `.github/skills/implement/SKILL.md`
  - Add step to update maturity matrix at checkpoint items

- [x] 11. **AGENT: Update AGENTS.md** — `AGENTS.md`
  - Update test counts after new tests are added (defer until after items 12–13)

- [x] **── CHECKPOINT 1a: Documentation & process fixes complete ──**
  Verify: R1.1, R1.2, R1.3, R1.4, R1.5, R1.6, R1.7 all satisfied. Run `bash docs/scripts/check-links.sh` and `bash docs/scripts/check-frontmatter.sh`.

- [x] 12. **ACCEPTANCE TESTS: `with` block tests** — `proto/ish-tests/error_handling/`
  - `with` block basic usage (resource acquired and released)
  - `with` block with error (resource still released on throw)
  - Nested `with` blocks
  - `with` block combined with `defer`

- [x] 13. **CODE: Implement ledger engine** — new `proto/ish-vm/src/ledger/` module (or new `proto/ish-ledger/` crate)
  - `FeatureState` enum: dimension-specific states (e.g., `Optional`/`Required` for annotations, `Runtime`/`Build` for audit)
  - `Standard` struct: name, optional parent, feature map (feature name → feature state)
  - `StandardRegistry`: register/lookup standards, resolve inheritance chain
  - `EntryType` struct: name, optional parent, required properties with types
  - `EntryTypeRegistry`: register/lookup entry types, validate entries against types
  - `AuditResult` enum: `Pass`, `AutoFix(Vec<Action>)`, `Discrepancy(DiscrepancyReport)`
  - Stateless `audit_statement()` function: given statement + active feature states + entries → `AuditResult`
  - Pre-register built-in standards: `streamlined`, `cautious`, `rigorous` with appropriate feature maps
  - Pre-register built-in entry types: `Error` (requires `message: String`), `CodedError` (extends `Error`, requires `code: String`), `SystemError` (extends `CodedError`), `Mutable`, `Type`, `Open`, `Closed`

- [x] 14. **CODE: Implement VM integration** — `proto/ish-vm/src/`
  - Standard scope stack: push on `@standard[name]`, pop on scope exit
  - Entry store: attach entries to values in the environment, query entries
  - Audit bridge: before statement execution, call `audit_statement()`, apply auto-fixes, report discrepancies as system errors

- [x] 15. **UNIT TESTS: Ledger engine tests** — alongside ledger code
  - StandardRegistry: registration, lookup, inheritance resolution
  - EntryTypeRegistry: registration, lookup, entry validation
  - audit_statement: pass, auto-fix, and discrepancy cases
  - Built-in standards and entry types present at startup

- [x] 16. **ACCEPTANCE TESTS: Ledger runtime** — `proto/ish-tests/assurance_ledger/`
  - Define and apply a custom standard
  - Query active feature state
  - Entry type definition and validation
  - Built-in standards available at startup
  - Standard inheritance (extends)
  - Standard scope (push/pop)

- [x] 17. **AGENT: Update AGENTS.md test counts** — `AGENTS.md`
  - Update counts to reflect new `with` block and ledger tests

- [x] **── CHECKPOINT 1b: Phase 1 complete ──**
  Verify: R1.8–R1.12 satisfied. Run `cd proto && cargo test --workspace` and `cd proto && bash ish-tests/run_all.sh`. All tests pass.

---

### Phase 2 — Types

- [x] 18. **SPEC: Comprehensive rewrite of types.md** — `docs/spec/types.md`
  - Correct open/closed defaults: type declarations are *indeterminate*, object literals are *closed* (Decision 16)
  - Replace literal types section with value-entry model (Decision 17): actual-value entries, possible-values entries, allowed-values entries, type declarations as entry hooks
  - Refine generic types: variance structurally determined (Decision 15), note open syntax questions
  - Add intersection types section (Decision 19): semantics, object intersection, primitive intersection → `never`, conflicting property recursion
  - Update type narrowing section to cross-reference ledger behavior (Decision 20)
  - Add deferred type inference for unannotated bindings (Decision 18)
  - Clarify terminology per Decision 22: type vs. entry vs. type declaration
  - Relocate or cross-reference features that are actually assurance ledger features (Decision 11): null safety, mutability, overflow, numeric precision, implicit conversions
  - Move error handling content to errors.md (placeholder reference until Phase 3)
  - Update all function names to `snake_case`
  - **This is a comprehensive rewrite, not a patch — restructure the entire document in light of the new conceptual model.**

- [x] 19. **SPEC: Comprehensive rewrite of assurance-ledger.md** — `docs/spec/assurance-ledger.md`
  - Update feature state naming to two-dimension model (Decision 14): `type_annotations` (`optional`/`required`), `type_audit` (`runtime`/`build`)
  - Add `Open` and `Closed` entry types (Decision 16)
  - Document type checking as a ledger feature (Decision 5): the VM checks type compatibility on every statement, the ledger maintains entries
  - Document type narrowing as entry maintenance (Decision 20): after every statement, the ledger produces revised entry sets; narrowing is the natural consequence of maintaining type entries through control flow
  - Add Error/CodedError/SystemError hierarchy as built-in entry types (Decision 3) — at minimum define them, full error semantics in errors.md
  - Document value entries (Decision 17): actual-value, possible-values, allowed-values
  - Add features relocated from types spec (Decision 11): null safety, mutability, overflow, etc.
  - **This is a comprehensive rewrite — restructure to reflect the ledger's central role in the language.**

- [x] 20. **ARCHITECTURE: Update vm.md** — `docs/architecture/vm.md`
  - Document ledger runtime architecture: engine vs. VM integration
  - Document entry tracking on values
  - Document audit bridge (statement → audit → execute → update entries)
  - Document standard scope stack
  - Update builtins section for naming changes

- [x] 21. **USER GUIDE: Update types** — `docs/user-guide/types.md`
  - Update for value-entry model, open/closed semantics, intersection types
  - Ensure naming uses `snake_case` conventions

- [x] 22. **USER GUIDE: Update assurance levels** — `docs/user-guide/assurance-levels.md`
  - Update for two-dimension feature state naming
  - Document type checking as a ledger feature

- [x] 23. **AI GUIDE: Update for type system changes** — `docs/ai-guide/` (orientation.md, patterns.md, playbooks as needed)
  - Update for value-entry model, open/closed semantics, ledger-centric type checking

- [x] **── CHECKPOINT 2a: Type specifications and documentation complete ──**
  Verify: R2.1, R2.2 satisfied. Run link and frontmatter checks.

- [x] 24. **CODE: Implement type compatibility checking in ledger** — ledger engine module
  - Simple type matching (by name)
  - Union type matching (value matches any member)
  - Optional type matching (inner type or null)
  - Object structural matching (required properties present with compatible types)
  - Function type matching (param count, param types, return type)
  - List type matching (element type)
  - Tuple type matching (position by position)
  - Intersection type matching (satisfies all constituent types; conflicting properties → `never`)

- [x] 25. **CODE: Wire type audit into ledger** — VM integration
  - On assignment: audit that value type is compatible with declared type annotation (if present)
  - On function call: audit that argument types match parameter types
  - On return: audit that return value matches declared return type
  - Respect `type_annotations` feature state: if `required` and annotation missing → discrepancy

- [x] 26. **CODE: Implement type narrowing** — ledger engine + VM integration
  - Entry set maintenance: save entries at branch point, create branch-specific entries
  - Narrowing rules: `is_type()` narrows type entry in true branch; null comparison removes null from type entry
  - Branch merge: on convergence, union entry sets from both branches
  - Wire into interpreter control flow: if/else, while, for

- [x] 27. **UNIT TESTS: Type checking and narrowing** — alongside ledger code
  - Type compatibility for each type category
  - Type audit on assignment, call, return
  - Narrowing: is_type, null exclusion, restoration, nested

- [x] 28. **ACCEPTANCE TESTS: Type checking** — `proto/ish-tests/type_checking/`
  - Correct/incorrect type annotations under `cautious` standard
  - Function parameter/return type checking
  - Union, optional, object structural matching
  - Intersection type: object intersection, primitive → never

- [x] 29. **ACCEPTANCE TESTS: Type narrowing** — `proto/ish-tests/type_narrowing/`
  - `is_type()` narrowing in if branch
  - Null exclusion narrowing
  - Entry restoration after branch exit
  - Nested narrowing

- [x] **── CHECKPOINT 2b: Phase 2 complete ──**
  Verify: R2.3–R2.7 satisfied. Run `cd proto && cargo test --workspace` and `cd proto && bash ish-tests/run_all.sh`. All tests pass.

---

### Phase 3 — Errors

- [x] 30. **SPEC: Create errors.md** — `docs/spec/errors.md`
  - Error/CodedError/SystemError entry hierarchy with domain subtypes
  - throw/catch/finally/defer semantics (incorporate from types.md and existing error-handling proposal)
  - Ledger integration: throw audit (auto-add Error entry if qualifies, discrepancy if not)
  - Error union types in function signatures
  - `undeclared_errors` feature states
  - Link to error catalog (`docs/errors/INDEX.md`)

- [x] 31. **SPEC: Move error content from types.md** — `docs/spec/types.md`
  - Remove error handling section from types.md
  - Replace with cross-reference to errors.md

- [x] 32. **SPEC: Update assurance-ledger.md for error entries** — `docs/spec/assurance-ledger.md`
  - Ensure Error/CodedError/SystemError/domain subtypes are fully specified as entry types
  - Document throw audit semantics

- [x] 33. **ARCHITECTURE: Update vm.md for error model** — `docs/architecture/vm.md`
  - Document replacement of RuntimeError with entry-based errors
  - Document throw audit flow

- [x] 34. **USER GUIDE: Update error handling** — `docs/user-guide/error-handling.md`
  - Update for entry-based error model
  - Document Error/CodedError hierarchy for user errors
  - Remove references to `new_error()`
  - Add `error_code()` documentation

- [x] 35. **AI GUIDE: Update for error model** — `docs/ai-guide/` (relevant files)
  - Update error handling patterns and antipatterns for entry-based model

- [x] 36. **ERROR CATALOG: Expand and restructure** — `docs/errors/INDEX.md`
  - Audit all error creation sites in interpreter and builtins
  - Categorize into existing (E001–E006) or new codes
  - Add domain subtype column (FileError, TypeError, ArgumentError, etc.)
  - Add hierarchy column showing entry type chain
  - Add agent maintenance note: "update catalog when adding new error conditions"

- [x] **── CHECKPOINT 3a: Error specifications and documentation complete ──**
  Verify: R3.1, R3.2, R3.8 satisfied. Run link and frontmatter checks.

- [x] 37. **CODE: Register domain error entry types** — ledger engine
  - Add built-in domain subtypes: `FileError`, `FileNotFoundError`, `PermissionError`, `TypeError`, `ArgumentError`, etc.
  - Each leaf has a specific error code mapping

- [x] 38. **CODE: Replace RuntimeError with entry-based errors** — `proto/ish-vm/src/`
  - Interpreter creates ish objects with `SystemError` entries instead of Rust `RuntimeError` structs
  - Thrown values carry Error/CodedError/SystemError entries
  - Catch/propagation works with entry-based error objects

- [x] 39. **CODE: Implement throw audit** — ledger engine + VM integration
  - On `throw expr`: evaluate expr, then audit:
    - Object with `message: String` and no Error entry → auto-add Error entry
    - Object with `message: String` + `code: String` and no CodedError entry → auto-add CodedError entry
    - Object without `message: String` → discrepancy → throw SystemError

- [x] 40. **CODE: Update error builtins** — `proto/ish-vm/src/`
  - Remove `new_error()` builtin
  - Update `is_error()` to check for `Error` entry (not `__is_error__`)
  - Update `error_message()` to read `message` property
  - Add `error_code()` builtin: reads `code` property (null if not CodedError)

- [x] 41. **CODE: Update all error creation sites** — `proto/ish-vm/src/`
  - Every place the interpreter creates an error must produce a SystemError object with an appropriate well-known error code
  - Map each site to the correct domain subtype

- [x] 42. **UNIT TESTS: Error model** — alongside error code
  - Entry-based error creation and identity
  - Throw audit: auto-add, discrepancy
  - Updated builtins: is_error, error_message, error_code

- [x] 43. **ACCEPTANCE TESTS: Error handling** — `proto/ish-tests/error_handling/`
  - Throw object with message → auto-gets Error entry
  - Throw object with message + code → auto-gets CodedError entry
  - Throw object without message → SystemError discrepancy
  - `is_error()` checks Error entry
  - `error_message()` reads message property
  - `error_code()` reads code property
  - At least one test per error code in catalog

- [x] **── CHECKPOINT 3b: Phase 3 complete ──**
  Verify: R3.3–R3.9 satisfied. Run `cd proto && cargo test --workspace` and `cd proto && bash ish-tests/run_all.sh`. All tests pass.

---

### Finalization

- [x] 44. **AGENT: Final AGENTS.md update** — `AGENTS.md`
  - Update all test counts
  - Add error catalog maintenance note: "update docs/errors/INDEX.md when adding new error conditions"

- [x] 45. **ROADMAP: Set to completed** — `docs/project/roadmap.md`
  - Move "Types, errors, and assurance ledger consistency" to Completed

- [x] 46. **MATURITY: Final update** — `docs/project/maturity.md`
  - Update all affected rows to reflect final implementation state

- [x] 47. **HISTORY: Update history index** — `docs/project/history/INDEX.md`
  - Ensure entry for this proposal is current and reflects acceptance

- [x] 48. **INDEX: Update spec index** — `docs/spec/INDEX.md`
  - Add `errors.md` entry

- [x] 49. **INDEX: Update plans index** — `docs/project/plans/INDEX.md`
  - Add this plan

- [x] 50. **INDEX: Update `## Referenced by` sections** — all modified files
  - Verify and update Referenced by sections in every file changed during implementation

- [x] **── FINAL CHECKPOINT ──**
  Run full test suite. Run `bash docs/scripts/check-links.sh`, `bash docs/scripts/check-frontmatter.sh`, `bash docs/scripts/check-glossary.sh`. Verify all requirements R1.1–R3.9 satisfied.

---

## Reference

### Error Codes (Current)
E001–E006 are currently defined in `docs/errors/INDEX.md`: Unhandled throw, Division by zero, Argument count mismatch, Type mismatch, Undefined variable, Not callable. All currently implemented as Rust `RuntimeError` structs.

### Current Function Names to Change
- `isType(t, i)` → `is_type(value, type)` (in `docs/spec/types.md`)
- `validate(t, i)` → `validate(type, value)` (in `docs/spec/types.md`)
- `new_error()` → removed (Phase 3)

### Current Feature State Names to Replace
- `optional` / `live` / `pre` → `type_annotations(optional|required)` + `type_audit(runtime|build)`

### Built-in Entry Types (Complete List)
Pre-registered at startup:
- `Type` — structural type entry
- `Mutable` — mutability tracking
- `Open` — type is open to extra properties
- `Closed` — type is closed to extra properties
- `Error` — requires `message: String`
- `CodedError` extends `Error` — requires `code: String`
- `SystemError` extends `CodedError` — code must be well-known ish error code
- Domain subtypes: `FileError`, `FileNotFoundError`, `PermissionError`, `TypeError`, `ArgumentError`, etc.

### Built-in Standards
- `streamlined` — minimal enforcement (all features optional/runtime)
- `cautious` — moderate enforcement (annotations required for public APIs)
- `rigorous` — maximum enforcement (all annotations required, build-time audit)

### Hybrid Datatype Location
- AST (`ish-ast`): Standard, EntryType, FeatureState definitions — immutable source-level declarations
- Ledger module: AuditResult, audit_statement(), StandardRegistry, EntryTypeRegistry — behavioral logic
- VM (`ish-vm`): scope stack, entry store, audit bridge — runtime wiring

### Open/Closed Defaults
- Type declarations: **indeterminate** (neither open nor closed)
- Object literals: **closed** by default
- Explicit annotations: `@[open]` or `@[closed]`

### Value Entry Taxonomy
- **Actual-value entry** (live audit): concrete runtime value
- **Possible-values entry** (pre audit): set of values across execution paths
- **Allowed-values entry**: explicit constraint on permitted values

---

## Referenced by

- [docs/project/plans/INDEX.md](INDEX.md)
- [docs/project/proposals/types-errors-assurance-consistency.md](../proposals/types-errors-assurance-consistency.md)
