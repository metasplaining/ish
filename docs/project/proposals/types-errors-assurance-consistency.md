---
title: "Proposal: Types, Errors, and Assurance Ledger — Consistency Audit"
category: proposal
audience: [human-dev, ai-agent]
status: accepted
last-verified: 2026-03-19
depends-on:
  - docs/spec/types.md
  - docs/spec/assurance-ledger.md
  - docs/spec/syntax.md
  - docs/errors/INDEX.md
  - docs/project/maturity.md
  - docs/project/roadmap.md
  - docs/project/proposals/error-handling.md
  - docs/project/proposals/assurance-ledger-syntax.md
  - docs/project/proposals/language-syntax.md
  - CONTRIBUTING.md
---

# Proposal: Types, Errors, and Assurance Ledger — Consistency Audit

*Generated from [types-errors-assurance-consistency.md](../rfp/types-errors-assurance-consistency.md) on 2026-03-19. Accepted 2026-03-19.*

---

## Decision Register

All decisions made during design, consolidated here as the authoritative reference.

| # | Decision | Outcome |
|---|----------|---------|
| 1 | Scope of this audit | All 10 original in-scope items plus generic types, type narrowing, literal types, intersection types, and custom type guards. Specifications refined with open questions resolved. |
| 2 | Maturity.md and roadmap.md corrections | Yes. Add maturity matrix to authority order and ensure skills maintain it. |
| 3 | Error representation model | Errors are ordinary objects with an `Error` entry (requires `message: String`). `CodedError` extends `Error` (requires `code: String`). `SystemError` extends `CodedError` (restricted to well-known ish error codes). Error hierarchy with domain subtypes (e.g., `FileError`, `FileNotFoundError`). `RuntimeError` eliminated. `new_error` removed. |
| 4 | Throw restriction enforcement | Only objects with the `Error` entry can be thrown. Enforced via assurance ledger: `throw` audits its argument for the `Error` entry, auto-adding it if the object qualifies (has `message: String`), or raising a system error discrepancy if it doesn't. |
| 5 | Type annotation enforcement | Live-audit type checking integrated with the assurance ledger. The VM always checks type compatibility on every statement execution. Build features in dependency order, interleaving as needed. |
| 6 | Naming conventions | `snake_case` for functions/variables. `PascalCase` for types and entry types. `SCREAMING_SNAKE_CASE` for constants. `snake_case` for modules and standards. |
| 7 | Assurance ledger runtime level | Foundation + queries. Clean VM/ledger separation. Stateless live-audit logic. |
| 8 | `with` block tests | Yes. |
| 9 | Error catalog approach | Audit all error paths. Structure docs for agent maintenance. |
| 10 | Implementation phasing | Three phases: foundations, types, errors. Errors last (use type and ledger mechanisms). |
| 11 | Type features and the assurance ledger | Many type-spec features (mutability, null safety, overflow, etc.) are actually assurance ledger features. Relocate or cross-reference. |
| 12 | Error spec visibility | Error handling gets its own spec file and spec index entry. |
| 13 | Ledger datatype location | Hybrid: AST defines what standards/entries *are* (immutable source-level declarations). A separate `ish-ledger` module defines what the auditor *does* with them (behavioral logic). `ish-vm` wires them together. |
| 14 | Feature state naming | Separate dimensions: `type_annotations` (`optional` / `required`) and `type_audit` (`runtime` / `build`). Replaces confusing `optional`/`live`/`pre` naming. |
| 15 | Variance in structural type system | Variance is largely moot in ish's structural type system. `Cat` (which has all properties of `Animal`) is always assignable to `Animal` unless `Animal` is closed. Invariant by default; structural subtyping handles the rest. |
| 16 | Open/closed type semantics | Type declarations (`type Cat = ...`) are *indeterminate* — neither open nor closed by default. May be inferred or coerced to either. Object literals are *closed* by default. Open/closed must be explicitly annotated. The assurance ledger combines open/closed entries from type annotations and literal values. |
| 17 | Value entries replace literal types | Values have both a structural type and behavioral entries. `let x = 5` has type `int` (inferrable to any integer type later) with an actual-value entry of `5`. Literal types are not needed — their use cases are handled by value entries (actual/possible/allowed). Type declarations also serve as hooks for behavioral entries. |
| 18 | Deferred type inference for unannotated bindings | Unannotated bindings wait for type inference. In `let x = 5`, the interpreter retains the string literal `"5"` and defers parsing/conversion until the actual type is inferred. The compiler infers during pre-audit. If no type can be inferred, the compiler widens. |
| 19 | Intersection types | Full intersection types. Conflicting property types produce `never`. |
| 20 | Type narrowing | Full control-flow narrowing. This is an assurance ledger behavior: after every statement, the ledger creates a revised entry set for every value. Structural type is one entry among many that gets maintained. |
| 21 | Custom type guards | Deferred. Built-in `is_type()` covers the immediate need. |
| 22 | Terminology: types vs. entries | Type declarations in ish are not pure structural declarations — they also serve as hooks for behavioral entries. This may cause ambiguity and requires careful terminology work. |

---

## Part 1: Audit Findings

This section documents every inconsistency found between specifications, documentation, implementation, and tests.

### 1.1 Maturity Matrix Inaccuracies

| Feature | Matrix Says | Actual State | Discrepancy |
|---------|-------------|--------------|-------------|
| Error handling | Designed: ❌, Spec: ❌, Proto: ❌, Tests: ❌ | Designed: ✅, Spec: partial, Proto: ✅, Tests: partial (16 assertions) | All four columns wrong |
| Assurance Ledger | Designed: ✅, Spec: ✅, Proto: ❌ | Proto: partial (parser + AST nodes, interpreter ignores) | Proto understated |
| Syntax / grammar | Designed: ❌, Spec: ❌, Proto: ❌, Tests: ❌ | All ✅ or partial | All four columns wrong |
| Parser | Designed: ❌, Spec: ❌, Proto: ❌, Tests: ❌ | Designed: ✅, Spec: partial, Proto: ✅, Tests: partial | All four columns wrong |

### 1.2 Roadmap Inaccuracies

"Error handling design" listed as In Progress but an accepted proposal exists with working prototype.

### 1.3 Authority Order Gap

Maturity matrix not in the authority order (only in change protocol section of CONTRIBUTING.md). Skills do not maintain it.

### 1.4 Spec Index Gap

No entry for error handling in [docs/spec/INDEX.md](../../spec/INDEX.md).

### 1.5 Types — Specification vs. Implementation Gaps

**Consistent**: Primitive types, objects, lists, functions — all implemented and tested.

**Parsed but not enforced**: Generic types, union types, tuple types, optional types, all type annotations, type aliases.

**Not parsed or implemented**: Intersection types, type narrowing, custom type guards, `validate()`, open/closed distinction, property mutability, numeric config, conversion rules, higher-kinded types, Type metatype.

**Type features that are assurance ledger features** (per Decision 11): null safety, mutability, overflow, numeric precision, implicit conversions, type annotation enforcement.

### 1.6 Errors — Specification vs. Implementation Gaps

**Consistent**: throw, try/catch/finally, defer, `new_error`/`is_error`/`error_message`, propagation, re-throw, exit codes.

**Not implemented**: Error codes, error categories, type-based catch, error union types, `undeclared_errors`, stack traces, custom error types. `with` blocks implemented but untested.

**Behavioral gaps**: Prototype allows throwing any value (should be Error-entry objects only). `RuntimeError` is a Rust struct (should be eliminated). `new_error()` to be removed.

### 1.7 Assurance Ledger — Specification vs. Implementation Gaps

All syntax parsed (standards, entries, entry types), interpreter ignores everything. Zero runtime behavior. Zero acceptance tests.

### 1.8 Cross-Cutting Inconsistencies

| Inconsistency | Resolution |
|----------------|-----------|
| `isType` naming mismatch | Resolved: `snake_case` convention adopted (Decision 6). Spec updates to `is_type`. |
| `validate` missing | Spec defines it; impl needed. |
| Throw restriction | Resolved: enforce via ledger (Decision 4). |
| Error identity via `__is_error__` | Resolved: replace with `Error` entry (Decision 3). |
| Type annotations decorative | Resolved: implement live-audit (Decision 5). |
| `with` blocks untested | Resolved: add tests (Decision 8). |
| Maturity matrix stale | Resolved: fix and add to authority order (Decision 2). |
| Errors missing from spec index | Resolved: create errors.md spec (Decision 12). |

---

## Part 2: Scope and Phasing

### In Scope

**Phase 1 — Foundations:**
1. Maturity matrix and roadmap corrections (including authority order fix)
2. Naming conventions for ish code
3. Assurance ledger runtime (foundation + queries + stateless live-audit)
4. `with` block tests

**Phase 2 — Types:**
5. Type annotation live-audit enforcement (integrated with ledger)
6. Generic types — specification refinement
7. Value entries (replacing literal types) — specification
8. Intersection types — specification and implementation
9. Type narrowing — specification and implementation
10. Open/closed type semantics — specification refinement

**Phase 3 — Errors:**
11. Error representation overhaul (Error/CodedError/SystemError entry hierarchy)
12. Throw restriction enforcement (via ledger)
13. Error catalog expansion
14. Error handling spec consolidation

### Out of Scope

- Pre-audit implementation (future — requires static analysis)
- Custom type guards (deferred per Decision 21)
- Memory model, polymorphism strategy (placeholder status)
- Pattern matching (not yet designed)
- Higher-kinded types, Type metatype

---

## Part 3: Feature Analysis

### Feature 1: Maturity Matrix, Roadmap, and Authority Order

#### Proposed Implementation

**Maturity matrix corrections** — update [docs/project/maturity.md](../../project/maturity.md):

| Feature | Designed | Spec Written | Prototyped | Tested | Stable |
|---------|----------|-------------|------------|--------|--------|
| Error handling | ✅ | partial | ✅ | partial | ❌ |
| Syntax / grammar | ✅ | ✅ | ✅ | partial | ❌ |
| Parser | ✅ | partial | ✅ | partial | ❌ |
| Assurance Ledger | ✅ | ✅ | partial | ❌ | ❌ |

**Roadmap corrections** — update [docs/project/roadmap.md](../../project/roadmap.md):
- Move "Error handling design" to Completed (note: "design complete; implementation pending entry-based error model")
- Add "Types, errors, and assurance ledger consistency" to In Progress

**Authority order** — add maturity matrix to the authority order in both [CONTRIBUTING.md](../../../CONTRIBUTING.md) and [.github/copilot-instructions.md](../../../.github/copilot-instructions.md):

1. GLOSSARY.md
2. Roadmap (status → "in progress")
3. **Maturity matrix (update affected rows)**
4. Specification docs
5. Architecture docs
6. User guide / AI guide
7. Agent documentation
8. Acceptance tests
9. Code
10. Unit tests
11. Roadmap (status → "completed")
12. **Maturity matrix (update affected rows)**
13. History
14. Index files

**Skill updates** — the `/implement` skill should include a step to update the maturity matrix at checkpoints.

---

### Feature 2: Naming Conventions for ish Code

Per Decision 6, ish adopts `snake_case` conventions.

#### Adopted Conventions

| Category | Convention | Examples |
|----------|-----------|---------|
| Variables | `snake_case` | `my_value`, `user_name` |
| Functions | `snake_case` | `get_user`, `is_type`, `to_string` |
| Types | `PascalCase` | `String`, `List`, `MyObject`, `Error` |
| Constants | `SCREAMING_SNAKE_CASE` | `MAX_SIZE`, `DEFAULT_TIMEOUT` |
| Modules | `snake_case` | `file_utils`, `http_client` |
| Entry types | `PascalCase` | `Error`, `SystemError`, `Mutable` |
| Standards | `snake_case` | `streamlined`, `cautious`, `rigorous` |
| Keywords | `lowercase` | `fn`, `let`, `mut`, `if`, `for`, `while`, `throw`, `try`, `catch` |

#### Rationale

All 45 existing builtins use `snake_case`. Rust (implementation language) uses `snake_case`. Shell integration is naturally `snake_case`. Both Rust and Python use this pattern. `PascalCase` for types is universal across all major languages. This avoids renaming existing code and is internally consistent.

#### Implementation

1. Update `docs/spec/types.md` — change `isType(t, i)` to `is_type(value, type)`, `validate(t, i)` to `validate(type, value)`.
2. Add naming convention section to `docs/spec/syntax.md`.
3. Update user guide and AI guide with naming conventions (primary audience for this decision).
4. No code changes needed — prototype already uses these conventions.

---

### Feature 3: Assurance Ledger Runtime

#### Architecture

The ledger runtime has two components with clean separation:

**Ledger Engine** (stateless, easily unit-tested):
- Stateless audit functions: given a statement + active feature states + entries → audit result (pass, auto-fix, discrepancy)
- Entry type registry: stores entry type definitions
- Standard registry: stores standard definitions with feature state maps

**VM Integration** (in `ish-vm`):
- Standard scope stack: tracks active standard per scope
- Entry store: attaches entries to values, queries them
- Audit bridge: calls engine per statement, applies results

#### Datatype Location (Decision 13)

Standards, entry types, and entries are immutable. They cannot be mutated at runtime or have arbitrary values injected. They must be stable at module/function declaration time. Every possible entry state must be inferrable from source code for pre-audit.

The adopted hybrid approach:
- **AST (`ish-ast`)**: Entry types and standards metadata — inherently source-level, immutable declarations — reside as AST-level types. Built-in standards and entry types are pre-registered AST-level constants.
- **Ledger module (`ish-ledger`)**: Auditing logic — how to check, what discrepancies to report — is behavioral and belongs in a separate module or crate.
- **VM (`ish-vm`)**: Wires the AST definitions and ledger logic together at runtime.

This gives: AST defines what standards/entries *are*, `ish-ledger` defines what the auditor *does* with them, and `ish-vm` wires them together.

#### Proposed Implementation

1. **Define `FeatureState` enum** (in AST or ledger): `Optional`, `Live`, `Pre`, plus feature-specific states.

2. **Define `Standard` struct**: name, optional parent, feature map.

3. **Define `EntryType` struct**: name, optional parent, required properties with types.

4. **Define `AuditResult` enum** (in ledger): `Pass`, `AutoFix(Vec<Action>)`, `Discrepancy(DiscrepancyReport)`.

5. **Implement `StandardRegistry`**: register/lookup standards, resolve inheritance.

6. **Implement `EntryTypeRegistry`**: register/lookup entry types, validate entries.

7. **Implement stateless `audit_statement()`**: core audit function, dispatches per statement kind.

8. **Pre-register built-in standards** (`streamlined`, `cautious`, `rigorous`) and built-in entry types (`Error`, `CodedError`, `SystemError`, `Mutable`, `Type`) at startup.

9. **Add standard scope stack** to interpreter: push on `@standard[name]`, pop on scope exit.

10. **Add entry tracking** to values in the environment.

11. **Wire audit bridge**: before execution, call `audit_statement`. Apply auto-fixes. Report discrepancies as system errors.

12. **Acceptance tests**:
    - Define and apply a custom standard
    - Query active feature state
    - Entry type definition and validation
    - Built-in standards available
    - Standard inheritance (extends)
    - Standard scope (push/pop)

---

### Feature 4: Type Annotation Live-Audit

#### Dependency

Requires Feature 3 (ledger runtime).

#### Feature State Naming (Decision 14)

Type checking has two independent dimensions:

- **Annotation requirement**: Are explicit type annotations required?
- **Audit timing**: When are type discrepancies detected?

These are separated into two features:
- `type_annotations`: `optional` | `required`
- `type_audit`: `runtime` | `build`

This replaces the confusing `optional`/`live`/`pre` naming. The assurance ledger supports compound feature states, so `types(annotations: required, audit: build)` is natural.

The VM always checks type compatibility on every statement (this is intrinsic). The question is what *additional* enforcement the ledger provides.

#### Semantics (per Decisions 5, 14, 20)

The VM always checks type compatibility on statement execution. This is inherent — assigning an incompatible value is always a discrepancy. The `types` feature controls whether annotations are *required* and *when* checking occurs:

- **annotations: optional, audit: runtime** — `let x = 5` is fine; `let x: String = 5` throws at runtime.
- **annotations: required, audit: runtime** — `let x = 5` is a discrepancy (no annotation); `let x: i32 = 5` is fine.
- **annotations: required, audit: build** — `let x = 5` is a build-time error; `let x: i32 = 5` is validated at build time.

The assurance ledger does more than check — it *maintains* the entry set (Decision 20). After every statement, a revised entry set is created. For type narrowing, this means:

```ish
let x: i32 | String = get_value()
// entries: { type: i32 | String }

if is_type(x, "i32") {
    // ledger narrows: entries for x become { type: i32 }
    let y = x + 1  // valid — type entry says i32
}
// entries for x revert to { type: i32 | String }
```

#### Type Compatibility Checking

- Simple types: match by name
- Union types: value matches if it matches any member
- Optional types: matches inner type or null
- Object types: structural matching (required properties present with compatible types)
- Function types: parameter count and type matching, return type matching
- List types: element type checking
- Tuple types: position-by-position matching
- Intersection types: value matches if it satisfies all constituent types

#### Proposed Implementation

1. Implement type compatibility checking in the ledger engine.
2. Wire type audit into the ledger for assignment, function call, and return statements.
3. Implement type narrowing: after `is_type()` checks and null comparisons, update entry sets.
4. Acceptance tests under `cautious` standard:
   - Correct/incorrect type annotations
   - Function parameter/return type checking
   - Union, optional, and object structural matching
   - Type narrowing after `is_type()` and `!= null`

---

### Feature 5: Generic Types — Specification Refinement

#### Variance (Decision 15)

In ish's structural type system, variance is largely moot. A `Cat` that has all the structural properties of `Animal` is always assignable to `Animal` by structural subtyping — no covariance annotation needed. The only exception is when `Animal` is declared closed, which prevents extra properties. In that case:

- A `List<Cat>` assigned to `List<Animal>` where `Animal` is open: works (Cat satisfies Animal structurally).
- A `List<Cat>` assigned to `List<Animal>` where `Animal` is closed: discrepancy (Cat has extra properties).

The ledger handles this through entry checking, not through variance annotations. Generic types are invariant by default; structural subtyping covers the cases that variance would handle in nominal type systems.

#### Decisions (resolved)

Variance is structurally determined, not annotation-driven. Generic type checking is live-audit initially; pre-audit when static analysis is implemented.

#### Open Questions Remaining

1. **Generic function syntax** — exact syntax for `fn foo<T>(x: T) -> T`.
2. **Constraints** — syntax for `T: Comparable` or `T extends { name: String }`.
3. **Defaults** — syntax for `T = String`.
4. **Inference** — when can generic type parameters be inferred from arguments?

These are syntax-level questions that should be resolved in the spec but do not block the ledger/type infrastructure work.

---

### Feature 6: Value Entries Replace Literal Types (Decision 17)

#### Conceptual Model

Every value in ish has both a *structural type* and *behavioral entries*. The structural type describes the shape of the value (int, string, object, etc.). The entries describe additional facts about the value (its actual value, whether it's mutable, its error status, etc.).

Where other languages use "literal types" (e.g., TypeScript's `type Direction = "north" | "south"`), ish uses *value entries*:

- **Actual-value entry** (live audit): the concrete value at runtime. `let x = 5` has type `int` and actual-value entry `5`.
- **Possible-values entry** (pre audit): the set of values that could exist on this code path. Pre-audit tracks all possible values across branches.
- **Allowed-values entry**: an explicit constraint on what values are permitted. `@[allowed_values("north", "south", "east", "west")]` or declared via a type alias.

#### Use Cases Supported

All use cases traditionally handled by literal types are handled by value entries:

1. **Discriminated unions**: A union `{status: "success", data: T} | {status: "error", message: String}` works because the actual/possible values of `status` narrow which variant applies.
2. **Exhaustive matching**: In a match/switch on a field with allowed-values entry, the ledger verifies all values are covered.
3. **Value constraint checking**: `@[allowed_values(1, 2, 3)]` on a parameter validates actual values against allowed values.
4. **Reachability analysis**: If possible-values entry for a condition is `{true}`, the false branch is unreachable.
5. **Never type inference**: If possible-values is empty, the type is `never`.
6. **Function overload dispatch** (future): the actual value of a literal argument determines which overload applies.
7. **Template composition** (future): value entries for string components could be combined.
8. **Const assertions**: equivalent to freezing a binding's actual-value entry so it doesn't widen.

#### Additional Use Cases from Other Languages

Research into TypeScript, Kotlin, Scala, and Haskell confirms that the value-entry model covers all major literal-type use cases:
- Discriminated union narrowing (TypeScript) — fits perfectly with value entries
- Exhaustive switch compilation (TypeScript, Kotlin) — fits perfectly
- Function overload keying on literal arguments (TypeScript) — fits perfectly
- Const assertions / as-const (TypeScript) — fits perfectly
- API response validation via discriminants (TypeScript) — fits perfectly
- Computed mapped type keys (TypeScript) — fits with value entries as keys
- Template literal composition (TypeScript) — partial fit, needs further specification
- Distributed conditional types (TypeScript) — partial fit, needs condition evaluation

No use cases were found that require literal-as-type semantics that value entries cannot address.

#### Type Declarations as Entry Hooks (Decision 22)

Type declarations in ish are not pure structural declarations. A `type Direction = "north" | "south" | "east" | "west"` declaration defines:
- A structural type (the union of string values)
- An allowed-values entry (constraining which strings are valid)

This dual nature means type declarations serve as hooks for behavioral entries. This creates potential ambiguity between the type system and the entry system. Careful terminology is needed:

- **Type**: the structural shape of a value (int, string, object with specific properties)
- **Entry**: a behavioral fact about a value (its actual value, its mutability, its error status, constraints)
- **Type declaration**: defines both a structural type AND associated entries

#### Deferred Type Inference (Decision 18)

In `let x = 5`, the interpreter does not immediately widen `5` to a specific integer type. Instead:
- The string literal `"5"` is retained.
- The type is recorded as "some integer type" (not yet resolved).
- When the type is needed (e.g., passed to a function expecting `i32`), inference resolves it.
- The compiler resolves during pre-audit.
- If no type can be inferred from usage, the compiler widens to a default integer type.

---

### Feature 7: Open/Closed Type Semantics (Decision 16)

#### Adopted Semantics

- **Type declarations** (`type Cat = Animal & { sound: String }`) are *indeterminate* — neither open nor closed by default. They can be inferred or coerced to either based on context.
- **Object literals** (`let x = { name: "Rex" }`) are *closed* by default.
- To make a type explicitly open or closed, annotate it: `@[open] type Foo = { ... }` or `@[closed] type Bar = { ... }`.
- A statement like `let x: Cat = { sound: "Meow" }` produces a closed Cat: the assurance ledger combines the indeterminate open/closed state from `Cat` with the closed state from the object literal, yielding a value with both entry sets.

This corrects the type spec which states that closed is the default for type declarations. Closed is the default only for *object literals*.

#### Implementation

1. Update `docs/spec/types.md` to correct open/closed defaults.
2. Add `Open` and `Closed` entry types to the ledger.
3. Object literals get `Closed` entry by default.
4. Type declarations get no open/closed entry (indeterminate).
5. Explicit `@[open]` / `@[closed]` annotations add the appropriate entry.
6. The ledger resolves open/closed during assignment by combining entries from both sides.

---

### Feature 8: Intersection Types (Decision 19)

Full intersection types adopted.

#### Semantics

- `type AB = A & B` — a value must satisfy both A and B.
- For objects: all properties from both types are required.
- For primitives: `int & string` is `never` (no value satisfies both).
- Conflicting property types are intersected recursively; if the intersection produces `never`, the whole intersection is `never`.

#### Implementation

1. Add intersection type to the type compatibility checker.
2. Parser already supports `&` — verify and extend if needed.
3. Add acceptance tests for object intersection, primitive intersection (→ never), and conflicting properties.

---

### Feature 9: Type Narrowing (Decision 20)

Full control-flow narrowing, implemented as an assurance ledger behavior.

#### Conceptual Model

The assurance ledger doesn't just *check* entries — it *maintains* them. After every statement, the ledger produces a revised entry set for every value in scope. Structural type is one entry that gets checked and maintained alongside others (actual value, mutability, etc.).

This means narrowing is not a special case bolted onto type checking — it's the natural consequence of the ledger maintaining entries through control flow:

- `if is_type(x, "int")` → in the true branch, the ledger updates x's type entry to `int`
- `if x != null` → in the true branch, null is removed from x's type entry
- After the branch, the ledger restores the pre-branch entries (or merges them if both branches converge)

#### Implementation

1. Implement entry set maintenance in the ledger engine:
   - Branch entry: save current entries, create branch-specific entries
   - Merge entry: on branch convergence, union the entry sets from both branches
   - Narrowing rules: `is_type` checks, null comparisons, truthiness checks
2. Wire into the interpreter's control flow (if/else, while, for).
3. Acceptance tests:
   - `is_type` narrowing in if branch
   - Null exclusion narrowing
   - Entry restoration after branch
   - Nested narrowing

---

### Feature 10: Error Representation Overhaul (Decisions 3, 4)

#### Error Entry Hierarchy

```
entry type Error {
    message: String
}

entry type CodedError extends Error {
    code: String
}

entry type SystemError extends CodedError {
    // code must be a well-known ish error code (E001, E002, etc.)
}
```

**Domain error subtypes** extending SystemError:
```
entry type FileError extends SystemError { }
entry type FileNotFoundError extends FileError { }
entry type PermissionError extends FileError { }
entry type TypeError extends SystemError { }
entry type ArgumentError extends SystemError { }
// ... etc.
```

Each leaf in the hierarchy has a specific error code. The error catalog enumerates all SystemError codes.

**User errors**: Users create errors by making objects with the `Error` entry:
```ish
throw { message: "Something went wrong" }           // Auto-gets Error entry
throw { message: "Bad input", code: "APP001" }      // Auto-gets CodedError entry
@[Error] throw { message: "Custom", extra: "data" } // Explicit Error entry
```

**System errors**: The language processor only throws `SystemError`s with well-known codes:
```ish
// Internal: SystemError { code: "E004", message: "Type mismatch: expected i32, got String" }
```

#### Throw Semantics (Decision 4)

1. `throw expr` evaluates `expr`.
2. Ledger audits the throw:
   - If value is an object with `message: String` and no `Error` entry → auto-add `Error` entry.
   - If value is an object with `message: String` and `code: String` and no `CodedError` entry → auto-add `CodedError` entry.
   - If value lacks `message: String` → discrepancy → throw SystemError about the discrepancy.
3. The thrown value (now guaranteed to have `Error` entry) propagates up the call stack.

#### Proposed Implementation

1. Define `Error`, `CodedError`, `SystemError`, and domain subtypes as built-in entry types.
2. Replace Rust `RuntimeError` — interpreter wraps/unwraps ish objects with `SystemError` entries for control flow.
3. Remove `new_error()` builtin.
4. Update `is_error()` — checks for `Error` entry.
5. Update `error_message()` — reads `message` property.
6. Add `error_code()` — reads `code` property (null if not CodedError).
7. Update throw handler with ledger audit logic.
8. Update all interpreter error sites to create SystemError objects.
9. Update acceptance tests.

---

### Feature 11: Error Catalog Expansion

1. Audit all error creation sites in interpreter and builtins.
2. Categorize into existing (E001–E006) or new codes.
3. Assign domain subtypes (FileError, TypeError, etc.) to each code.
4. Update [docs/errors/INDEX.md](../../errors/INDEX.md) with complete catalog including hierarchy column.
5. Add maintenance note in error catalog and AGENTS.md: "update catalog when adding new error conditions."
6. Add acceptance tests — at least one per error code.

---

### Feature 12: Error Handling Spec Consolidation

1. Create `docs/spec/errors.md`:
   - Error/CodedError/SystemError entry hierarchy
   - throw/catch/finally/defer semantics
   - Ledger integration for throw validation
   - Error union types in function signatures
   - `undeclared_errors` feature states
   - Link to error catalog
2. Move error content from `docs/spec/types.md` to `errors.md`.
3. Add `errors.md` to `docs/spec/INDEX.md`.

---

### Feature 13: `with` Block Tests

Add acceptance tests to `proto/ish-tests/error_handling/`:
1. `with` block basic usage (resource acquired and released)
2. `with` block with error (resource still released on throw)
3. Nested `with` blocks
4. `with` block combined with `defer`

---

## Part 4: Implementation Phasing

### Phase 1 — Foundations

| # | Item | Depends On |
|---|------|-----------|
| 1.1 | Fix maturity matrix | — |
| 1.2 | Fix roadmap | — |
| 1.3 | Add maturity matrix to authority order | — |
| 1.4 | Update implement skill for maturity | 1.3 |
| 1.5 | Add naming conventions to spec | — |
| 1.6 | Update spec function names to `snake_case` | 1.5 |
| 1.7 | Implement ledger engine (registries, audit function) | — |
| 1.8 | Implement VM integration (scope stack, entry tracking, audit bridge) | 1.7 |
| 1.9 | Add ledger acceptance tests | 1.8 |
| 1.10 | `with` block tests | — |

### Phase 2 — Types

| # | Item | Depends On |
|---|------|-----------|
| 2.1 | Refine generic types spec | 1.5 |
| 2.2 | Specify value entries (replacing literal types) | — |
| 2.3 | Specify intersection types | — |
| 2.4 | Specify type narrowing | — |
| 2.5 | Correct open/closed defaults in types spec | — |
| 2.6 | Implement type compatibility checking in ledger | 1.7 |
| 2.7 | Wire type audit into ledger | 2.6, 1.8 |
| 2.8 | Implement type narrowing (entry maintenance) | 2.7 |
| 2.9 | Add type checking acceptance tests | 2.7 |
| 2.10 | Add narrowing acceptance tests | 2.8 |

### Phase 3 — Errors

| # | Item | Depends On |
|---|------|-----------|
| 3.1 | Create errors.md spec | — |
| 3.2 | Define Error/CodedError/SystemError/domain entry types | 1.7 |
| 3.3 | Replace RuntimeError with entry-based errors | 3.2, 2.6 |
| 3.4 | Implement throw audit | 3.2, 1.8 |
| 3.5 | Remove `new_error`; update `is_error`/`error_message`; add `error_code` | 3.3 |
| 3.6 | Audit all error paths; expand catalog | 3.3 |
| 3.7 | Add error acceptance tests | 3.4, 3.5, 3.6 |

---

## Documentation Updates

| File | Reason |
|------|--------|
| [docs/project/maturity.md](../../project/maturity.md) | Fix inaccurate rows |
| [docs/project/roadmap.md](../../project/roadmap.md) | Fix error handling status; add consistency work |
| [CONTRIBUTING.md](../../../CONTRIBUTING.md) | Add maturity matrix to authority order |
| [.github/copilot-instructions.md](../../../.github/copilot-instructions.md) | Add maturity matrix to authority order |
| [.github/skills/implement/SKILL.md](../../../.github/skills/implement/SKILL.md) | Add maturity matrix update step |
| [docs/spec/types.md](../../spec/types.md) | Correct open/closed defaults; update naming; move errors to errors.md; add value-entry model; refine generics |
| [docs/spec/syntax.md](../../spec/syntax.md) | Add naming conventions section |
| [docs/spec/INDEX.md](../../spec/INDEX.md) | Add errors.md entry |
| [docs/spec/errors.md](../../spec/errors.md) | New — error handling specification |
| [docs/spec/assurance-ledger.md](../../spec/assurance-ledger.md) | Add Error/CodedError/SystemError/domain entry types; add Open/Closed entries |
| [docs/errors/INDEX.md](../../errors/INDEX.md) | Expand catalog; add hierarchy; add agent maintenance note |
| [docs/architecture/vm.md](../../architecture/vm.md) | Document ledger runtime, updated builtins, entry tracking |
| [AGENTS.md](../../../AGENTS.md) | Update test counts; add error catalog maintenance note |

Remember to update `## Referenced by` sections in all modified files.

When updating the types and assurance ledger specs, comprehensively fix the entire documents in light of the changes to the conceptual model made in this proposal — don't just patch the documents. Also update the user guide and AI guide.

---

## History Updates

- [x] Create `docs/project/history/2026-03-19-types-errors-assurance-consistency/` directory
- [x] Add `summary.md` with narrative prose
- [x] Save v1.md, v2.md
- [x] Save v3.md
- [x] Update `summary.md` with v3 narrative and acceptance
- [ ] Update [docs/project/history/INDEX.md](../../project/history/INDEX.md)

---

## Referenced by

- [docs/project/proposals/INDEX.md](INDEX.md)
- [docs/project/rfp/types-errors-assurance-consistency.md](../rfp/types-errors-assurance-consistency.md)
